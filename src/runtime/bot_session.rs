use futures::channel::mpsc::UnboundedSender;
use parking_lot::{Condvar, Mutex, RwLock};

use crate::bot::BotRunner;
use crate::config::SearchConfig;
use crate::protocol::sbp::SearchInfo;
use crate::runtime::bot_factory::BotInstance;
use crate::runtime::search_driver::{SearchDriver, SearchRunner, SearchState};
use crate::tetris::model::{Board, BoardSnapshot, DynamicBoard, Piece, Placement};

pub struct BotSession {
    state: Mutex<SearchState>,
    blocker: Condvar,
    runner: RwLock<Option<BotRunnerInstance>>,
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

    pub fn start(&self, initial_state: BotInstance) {
        let mut state = self.state.lock();
        state.reset_session_stats();
        *self.runner.write() = Some(BotRunnerInstance::from_bot(
            initial_state,
            &self.search_config,
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

    pub fn replace_board(&self, board: BoardSnapshot) {
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
        runner
            .as_mut()
            .map_or_else(Vec::new, BotRunnerInstance::drain_logs)
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

enum BotRunnerInstance {
    Standard(BotRunner<Board>),
    Dynamic(BotRunner<DynamicBoard>),
}

impl BotRunnerInstance {
    fn from_bot(bot: BotInstance, config: &SearchConfig, thread_local: bool) -> Self {
        match bot {
            BotInstance::Standard(bot) => BotRunnerInstance::Standard(BotRunner::from_rng_config(
                bot,
                &config.rng,
                thread_local,
            )),
            BotInstance::Dynamic(bot) => BotRunnerInstance::Dynamic(BotRunner::from_rng_config(
                bot,
                &config.rng,
                thread_local,
            )),
        }
    }

    fn suggest(&self) -> Vec<Placement> {
        match self {
            BotRunnerInstance::Standard(runner) => runner.suggest(),
            BotRunnerInstance::Dynamic(runner) => runner.suggest(),
        }
    }

    fn step(&self) -> crate::bot::Statistics {
        match self {
            BotRunnerInstance::Standard(runner) => runner.step(),
            BotRunnerInstance::Dynamic(runner) => runner.step(),
        }
    }

    fn run_for(&self, budget: crate::search::SearchBudget) -> crate::bot::Statistics {
        match self {
            BotRunnerInstance::Standard(runner) => runner.run_for(budget),
            BotRunnerInstance::Dynamic(runner) => runner.run_for(budget),
        }
    }

    fn advance(&mut self, mv: Placement) {
        match self {
            BotRunnerInstance::Standard(runner) => runner.advance(mv),
            BotRunnerInstance::Dynamic(runner) => runner.advance(mv),
        }
    }

    fn replace_board(&mut self, board: BoardSnapshot) {
        match self {
            BotRunnerInstance::Standard(runner) => {
                runner.replace_board(board.into_fixed::<10>().expect("active runner is width 10"));
            }
            BotRunnerInstance::Dynamic(runner) => {
                let width = runner.board_width();
                runner.replace_board(
                    board
                        .into_dynamic_width(width)
                        .expect("replacement board width matches active runner"),
                );
            }
        }
    }

    fn new_piece(&mut self, piece: Piece) {
        match self {
            BotRunnerInstance::Standard(runner) => runner.new_piece(piece),
            BotRunnerInstance::Dynamic(runner) => runner.new_piece(piece),
        }
    }

    fn drain_logs(&mut self) -> Vec<String> {
        match self {
            BotRunnerInstance::Standard(runner) => runner.drain_logs(),
            BotRunnerInstance::Dynamic(runner) => runner.drain_logs(),
        }
    }
}

impl SearchRunner for BotRunnerInstance {
    fn suggest(&self) -> Vec<Placement> {
        BotRunnerInstance::suggest(self)
    }

    fn step(&self) -> crate::bot::Statistics {
        BotRunnerInstance::step(self)
    }

    fn run_for(&self, budget: crate::search::SearchBudget) -> crate::bot::Statistics {
        BotRunnerInstance::run_for(self, budget)
    }
}
