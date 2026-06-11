use std::time::Instant;

use parking_lot::{Condvar, Mutex, RwLock};

use crate::bot::{Bot, Statistics};
use crate::protocol::sbp::SearchInfo;
use crate::tetris::model::{Piece, Placement};

pub struct BotSyncronizer {
    state: Mutex<State>,
    blocker: Condvar,
    bot: RwLock<Option<Bot>>,
}

impl BotSyncronizer {
    pub fn new() -> Self {
        BotSyncronizer {
            state: Mutex::new(State {
                stats: Default::default(),
                last_advance: Instant::now(),
                node_limit: u64::MAX,
                start: Instant::now(),
                nodes_since_start: 0,
            }),
            blocker: Condvar::new(),
            bot: RwLock::new(None),
        }
    }

    pub fn start(&self, initial_state: Bot) {
        let mut state = self.state.lock();
        state.reset_session_stats();
        *self.bot.write() = Some(initial_state);
        self.blocker.notify_all();
    }

    pub fn stop(&self) {
        let mut state = self.state.lock();
        state.reset_session_stats();
        *self.bot.write() = None;
        self.blocker.notify_all();
    }

    pub fn suggest(&self) -> Option<(Vec<Placement>, SearchInfo)> {
        let bot = self.bot.read();
        bot.as_ref().map(|bot| {
            let state = self.state.lock();
            let suggestion = bot.suggest();
            let info = SearchInfo {
                nodes: state.stats.nodes,
                nps: state.stats.nodes as f64 / state.last_advance.elapsed().as_secs_f64(),
                extra: format!(
                    "{:.1}% of selections expanded, overall speed: {:.1} Mnps",
                    state.stats.expansions as f64 / state.stats.selections as f64 * 100.0,
                    state.nodes_since_start as f64
                        / state.start.elapsed().as_secs_f64()
                        / 1_000_000.0
                ),
            };
            (suggestion, info)
        })
    }

    pub fn advance(&self, mv: Placement) {
        let mut state = self.state.lock();
        state.stats = Default::default();
        state.last_advance = Instant::now();
        let mut bot = self.bot.write();
        if let Some(bot) = &mut *bot {
            bot.advance(mv);
        }
        self.blocker.notify_all();
    }

    pub fn new_piece(&self, piece: Piece) {
        let mut bot = self.bot.write();
        if let Some(bot) = &mut *bot {
            bot.new_piece(piece);
        }
        self.blocker.notify_all();
    }

    pub fn drain_logs(&self) -> Vec<String> {
        let mut bot = self.bot.write();
        bot.as_mut().map_or_else(Vec::new, Bot::drain_logs)
    }

    pub fn work_loop(&self) {
        let mut state = self.state.lock();
        loop {
            if state.stats.nodes > state.node_limit {
                self.blocker.wait(&mut state);
                continue;
            }
            let bot_guard = self.bot.read();
            let bot = match &*bot_guard {
                Some(bot) => bot,
                None => {
                    drop(bot_guard);
                    self.blocker.wait(&mut state);
                    continue;
                }
            };

            drop(state);
            let new_stats = bot.do_work();
            drop(bot_guard);

            state = self.state.lock();
            state.stats.accumulate(new_stats);
            state.nodes_since_start += new_stats.nodes;
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct State {
    stats: Statistics,
    last_advance: Instant,
    node_limit: u64,
    start: Instant,
    nodes_since_start: u64,
}

impl State {
    fn reset_session_stats(&mut self) {
        let now = Instant::now();
        self.stats = Default::default();
        self.last_advance = now;
        self.start = now;
        self.nodes_since_start = 0;
    }
}
