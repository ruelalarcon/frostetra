use futures::channel::mpsc::UnboundedSender;
use parking_lot::{Condvar, Mutex, RwLock};

use crate::bot::{Bot, BotRunner};
use crate::config::SearchConfig;
use crate::protocol::sbp::SearchInfo;
use crate::runtime::search_driver::{SearchDriver, SearchState};
use crate::tetris::model::{Board, Piece, Placement};

pub struct BotSession {
    state: Mutex<SearchState>,
    blocker: Condvar,
    runner: RwLock<Option<BotRunner>>,
    driver: SearchDriver,
    search_config: SearchConfig,
    log_sender: UnboundedSender<String>,
}

impl BotSession {
    pub fn new(log_sender: UnboundedSender<String>, search_config: SearchConfig) -> Self {
        let driver = SearchDriver::from_config(&search_config);
        let node_limit = driver.node_limit();
        BotSession {
            state: Mutex::new(SearchState::new(node_limit)),
            blocker: Condvar::new(),
            runner: RwLock::new(None),
            driver,
            search_config,
            log_sender,
        }
    }

    pub fn starts_worker(&self) -> bool {
        self.driver.starts_worker()
    }

    pub fn start(&self, initial_state: Bot) {
        let mut state = self.state.lock();
        state.reset_session_stats();
        *self.runner.write() = Some(BotRunner::from_rng_config(
            initial_state,
            &self.search_config.rng,
            !self.driver.starts_worker(),
        ));
        self.blocker.notify_all();
    }

    pub fn stop(&self) {
        let mut state = self.state.lock();
        state.reset_session_stats();
        *self.runner.write() = None;
        self.blocker.notify_all();
    }

    pub fn suggest(&self) -> Option<(Vec<Placement>, SearchInfo)> {
        let runner = self.runner.read();
        runner.as_ref().map(|runner| {
            let mut state = self.state.lock();
            self.driver.suggest(runner, &mut state)
        })
    }

    pub fn advance(&self, mv: Placement) {
        let mut state = self.state.lock();
        state.reset_advance_stats();
        let mut runner = self.runner.write();
        if let Some(runner) = &mut *runner {
            runner.advance(mv);
        }
        self.blocker.notify_all();
    }

    pub fn replace_board(&self, board: Board) {
        let mut state = self.state.lock();
        state.reset_advance_stats();
        let mut runner = self.runner.write();
        if let Some(runner) = &mut *runner {
            runner.replace_board(board);
        }
        self.blocker.notify_all();
    }

    pub fn new_piece(&self, piece: Piece) {
        let mut runner = self.runner.write();
        if let Some(runner) = &mut *runner {
            runner.new_piece(piece);
        }
        self.blocker.notify_all();
    }

    pub fn drain_logs(&self) -> Vec<String> {
        let mut runner = self.runner.write();
        runner.as_mut().map_or_else(Vec::new, BotRunner::drain_logs)
    }

    pub fn work_loop(&self) {
        loop {
            {
                let mut state = self.state.lock();
                if state.stats.nodes >= state.node_limit {
                    self.blocker.wait(&mut state);
                    continue;
                }
            }

            let runner_guard = self.runner.read();
            let runner = match &*runner_guard {
                Some(runner) => runner,
                None => {
                    drop(runner_guard);
                    let mut state = self.state.lock();
                    self.blocker.wait(&mut state);
                    continue;
                }
            };

            let new_stats = runner.step();
            drop(runner_guard);

            let mut state = self.state.lock();
            state.accumulate(new_stats);
            if !state.logged_node_limit && state.stats.nodes >= state.node_limit {
                state.logged_node_limit = true;
                let _ = self.log_sender.unbounded_send(format!(
                    "node cap reached: {} nodes; max expanded depth: {} plies; background search paused until the next piece",
                    state.node_limit,
                    state.stats.max_depth
                ));
            }
            std::thread::yield_now();
        }
    }
}
