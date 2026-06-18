use crate::bot::{Bot, Statistics};
use crate::config::SearchRngConfig;
use crate::search::{SearchBudget, SearchContext};
use crate::tetris::model::{Piece, Placement};

pub struct BotRunner {
    bot: Bot,
    context: SearchContext,
}

impl BotRunner {
    pub fn from_seed(bot: Bot, seed: u64) -> Self {
        BotRunner {
            bot,
            context: SearchContext::from_seed(seed),
        }
    }

    pub fn from_entropy(bot: Bot) -> Self {
        BotRunner {
            bot,
            context: SearchContext::from_entropy(),
        }
    }

    pub fn from_rng_config(bot: Bot, config: &SearchRngConfig) -> Self {
        match config {
            SearchRngConfig::Entropy => Self::from_entropy(bot),
            SearchRngConfig::Seeded { seed } => Self::from_seed(bot, *seed),
        }
    }

    /// Search uses interior mutability in the DAG and RNG so background
    /// expansion can share a read lock with suggestions. Gameplay mutations
    /// (`advance`/`new_piece`) still require exclusive access to the runner.
    pub fn step(&self) -> Statistics {
        self.bot.step_search(&self.context)
    }

    pub fn run_for(&self, budget: SearchBudget) -> Statistics {
        match budget {
            SearchBudget::Iterations(iterations) => {
                let mut stats = Statistics::default();
                for _ in 0..iterations {
                    stats.accumulate(self.step());
                }
                stats
            }
            SearchBudget::Nodes(nodes) => {
                let mut stats = Statistics::default();
                while stats.nodes < nodes {
                    stats.accumulate(self.step());
                }
                stats
            }
        }
    }

    pub fn suggest(&self) -> Vec<Placement> {
        self.bot.suggest()
    }

    pub fn advance(&mut self, mv: Placement) {
        self.bot.advance(mv);
    }

    pub fn new_piece(&mut self, piece: Piece) {
        self.bot.new_piece(piece);
    }

    pub fn drain_logs(&mut self) -> Vec<String> {
        self.bot.drain_logs()
    }
}
