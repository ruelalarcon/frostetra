use std::time::Instant;

use crate::bot::{BotRunner, Statistics};
use crate::config::{SearchBudgetConfig, SearchConfig};
use crate::protocol::sbp::SearchInfo;
use crate::search::SearchBudget;
use crate::tetris::model::Placement;
use crate::tetris::movegen::MovegenBoard;

pub(crate) trait SearchRunner {
    fn suggest(&self) -> Vec<Placement>;
    fn step(&self) -> Statistics;
    fn run_for(&self, budget: SearchBudget) -> Statistics;
}

impl<B: MovegenBoard> SearchRunner for BotRunner<B> {
    fn suggest(&self) -> Vec<Placement> {
        BotRunner::suggest(self)
    }

    fn step(&self) -> Statistics {
        BotRunner::step(self)
    }

    fn run_for(&self, budget: SearchBudget) -> Statistics {
        BotRunner::run_for(self, budget)
    }
}

#[derive(Clone)]
pub enum SearchDriver {
    Background(BackgroundSearchDriver),
    Budgeted(BudgetedSearchDriver),
}

impl SearchDriver {
    pub fn from_config(config: &SearchConfig) -> Self {
        match &config.budget {
            SearchBudgetConfig::Background { node_limit } => {
                SearchDriver::Background(BackgroundSearchDriver {
                    node_limit: *node_limit,
                })
            }
            SearchBudgetConfig::IterationsPerSuggest { iterations } => {
                SearchDriver::Budgeted(BudgetedSearchDriver {
                    budget: SearchBudget::Iterations(*iterations),
                })
            }
            SearchBudgetConfig::NodesPerSuggest { nodes } => {
                SearchDriver::Budgeted(BudgetedSearchDriver {
                    budget: SearchBudget::Nodes(*nodes),
                })
            }
        }
    }

    pub fn starts_worker(&self) -> bool {
        matches!(self, SearchDriver::Background(_))
    }

    pub fn node_limit(&self) -> u64 {
        match self {
            SearchDriver::Background(driver) => driver.node_limit,
            SearchDriver::Budgeted(_) => 0,
        }
    }

    pub fn suggest(
        &self,
        runner: &impl SearchRunner,
        state: &mut SearchState,
    ) -> (Vec<Placement>, SearchInfo) {
        match self {
            SearchDriver::Background(driver) => driver.suggest(runner, state),
            SearchDriver::Budgeted(driver) => driver.suggest(runner, state),
        }
    }
}

#[derive(Clone)]
pub struct BackgroundSearchDriver {
    node_limit: u64,
}

impl BackgroundSearchDriver {
    fn suggest(
        &self,
        runner: &impl SearchRunner,
        state: &mut SearchState,
    ) -> (Vec<Placement>, SearchInfo) {
        let mut suggestion = runner.suggest();
        if suggestion.is_empty() {
            state.accumulate(runner.step());
            suggestion = runner.suggest();
        }

        (suggestion, state.search_info())
    }
}

#[derive(Clone)]
pub struct BudgetedSearchDriver {
    budget: SearchBudget,
}

impl BudgetedSearchDriver {
    fn suggest(
        &self,
        runner: &impl SearchRunner,
        state: &mut SearchState,
    ) -> (Vec<Placement>, SearchInfo) {
        state.accumulate(runner.run_for(self.budget));
        let suggestion = runner.suggest();
        (suggestion, state.search_info())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SearchState {
    pub stats: Statistics,
    pub last_advance: Instant,
    pub node_limit: u64,
    pub logged_node_limit: bool,
    pub start: Instant,
    pub nodes_since_start: u64,
}

impl SearchState {
    pub fn new(node_limit: u64) -> Self {
        SearchState {
            stats: Default::default(),
            last_advance: Instant::now(),
            node_limit,
            logged_node_limit: false,
            start: Instant::now(),
            nodes_since_start: 0,
        }
    }

    pub fn reset_session_stats(&mut self) {
        let now = Instant::now();
        self.stats = Default::default();
        self.logged_node_limit = false;
        self.last_advance = now;
        self.start = now;
        self.nodes_since_start = 0;
    }

    pub fn reset_advance_stats(&mut self) {
        self.stats = Default::default();
        self.last_advance = Instant::now();
    }

    pub fn accumulate(&mut self, stats: Statistics) {
        self.stats.accumulate(stats);
        self.nodes_since_start += stats.nodes;
    }

    pub fn search_info(&self) -> SearchInfo {
        let elapsed_since_advance = self.last_advance.elapsed().as_secs_f64();
        let elapsed_since_start = self.start.elapsed().as_secs_f64();
        let expansion_rate = if self.stats.selections == 0 {
            0.0
        } else {
            self.stats.expansions as f64 / self.stats.selections as f64 * 100.0
        };

        SearchInfo {
            nodes: self.stats.nodes,
            nps: self.stats.nodes as f64 / elapsed_since_advance,
            extra: format!(
                "{:.1}% of selections expanded, overall speed: {:.1} Mnps",
                expansion_rate,
                self.nodes_since_start as f64 / elapsed_since_start / 1_000_000.0
            ),
        }
    }
}
