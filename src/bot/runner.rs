use std::sync::OnceLock;
use std::thread::{self, ThreadId};

use crate::bot::{Bot, Statistics};
use crate::config::SearchRngConfig;
use crate::search::{SearchBudget, SearchContext};
use crate::tetris::model::{Board, BoardRepresentation, Piece, Placement};
use crate::tetris::movegen::MovegenBoard;

pub struct BotRunner<B: BoardRepresentation = Board> {
    bot: Bot<B>,
    context: SearchContext,
    thread_owner: Option<OnceLock<ThreadId>>,
}

impl<B: MovegenBoard> BotRunner<B> {
    pub fn from_seed(bot: Bot<B>, seed: u64, thread_local: bool) -> Self {
        BotRunner {
            bot,
            context: SearchContext::from_seed(seed),
            thread_owner: thread_local.then(OnceLock::new),
        }
    }

    pub fn from_entropy(bot: Bot<B>, thread_local: bool) -> Self {
        BotRunner {
            bot,
            context: SearchContext::from_entropy(),
            thread_owner: thread_local.then(OnceLock::new),
        }
    }

    pub fn from_rng_config(bot: Bot<B>, config: &SearchRngConfig, thread_local: bool) -> Self {
        match config {
            SearchRngConfig::Entropy => Self::from_entropy(bot, thread_local),
            SearchRngConfig::Seeded { seed } => Self::from_seed(bot, *seed, thread_local),
        }
    }

    /// Search uses interior mutability in the DAG and RNG so background
    /// expansion can share a read lock with suggestions. Gameplay mutations
    /// (`advance`/`new_piece`) still require exclusive access to the runner.
    pub fn step(&self) -> Statistics {
        self.assert_thread_owner();
        self.step_unchecked()
    }

    /// Runs one shared-DAG expansion using a caller-owned random stream.
    /// Background workers use this so they never contend on the runner's RNG.
    pub fn step_with_context(&self, context: &SearchContext) -> Statistics {
        self.assert_thread_owner();
        self.bot.step_search(context)
    }

    pub fn run_for(&self, budget: SearchBudget) -> Statistics {
        self.assert_thread_owner();
        match budget {
            SearchBudget::Iterations(iterations) => {
                let mut stats = Statistics::default();
                for _ in 0..iterations {
                    stats.accumulate(self.step_unchecked());
                }
                stats
            }
            SearchBudget::Nodes(nodes) => {
                let mut stats = Statistics::default();
                while stats.nodes < nodes {
                    stats.accumulate(self.step_unchecked());
                }
                stats
            }
        }
    }

    pub fn suggest(&self) -> Vec<Placement> {
        self.assert_thread_owner();
        self.bot.suggest()
    }

    pub fn advance(&mut self, mv: Placement) {
        self.assert_thread_owner();
        self.bot.advance(mv);
    }

    pub fn replace_board(&mut self, board: B) {
        self.assert_thread_owner();
        self.bot.replace_board(board);
    }

    pub fn board_width(&self) -> usize {
        self.bot.board_width()
    }

    pub fn new_piece(&mut self, piece: Piece) {
        self.assert_thread_owner();
        self.bot.new_piece(piece);
    }

    pub fn drain_logs(&mut self) -> Vec<String> {
        self.bot.drain_logs()
    }

    fn step_unchecked(&self) -> Statistics {
        self.bot.step_search(&self.context)
    }

    fn assert_thread_owner(&self) {
        let Some(owner) = &self.thread_owner else {
            return;
        };

        let current = thread::current().id();
        let expected = owner.get_or_init(|| current);
        assert_eq!(
            *expected, current,
            "thread-local BotRunner accessed from multiple threads"
        );
    }
}
