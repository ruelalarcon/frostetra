use std::num::NonZeroUsize;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct SearchConfig {
    pub rng: SearchRngConfig,
    pub budget: SearchBudgetConfig,
    pub threads: NonZeroUsize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounting_batch: Option<NonZeroUsize>,
}

impl SearchConfig {
    pub fn accounting_batch(&self) -> NonZeroUsize {
        self.accounting_batch
            .unwrap_or(NonZeroUsize::new(1).unwrap())
    }

    pub(super) fn merge(&mut self, override_config: SearchConfigOverride) {
        if let Some(rng) = override_config.rng {
            self.rng = rng;
        }
        if let Some(budget) = override_config.budget {
            self.budget = budget;
        }
        if let Some(threads) = override_config.threads {
            self.threads = threads;
        }
        if override_config.accounting_batch.is_some() {
            self.accounting_batch = override_config.accounting_batch;
        }
    }
}

impl<'de> Deserialize<'de> for SearchConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RequiredSearchConfig {
            rng: SearchRngConfig,
            budget: SearchBudgetConfig,
            threads: Option<NonZeroUsize>,
            accounting_batch: Option<NonZeroUsize>,
        }

        let config = RequiredSearchConfig::deserialize(deserializer)?;
        Ok(SearchConfig {
            rng: config.rng,
            budget: config.budget,
            threads: config.threads.unwrap_or(NonZeroUsize::new(1).unwrap()),
            accounting_batch: config.accounting_batch,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SearchConfigOverride {
    rng: Option<SearchRngConfig>,
    budget: Option<SearchBudgetConfig>,
    threads: Option<NonZeroUsize>,
    accounting_batch: Option<NonZeroUsize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum SearchRngConfig {
    Entropy,
    Seeded { seed: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum SearchBudgetConfig {
    Background { node_limit: u64 },
    IterationsPerSuggest { iterations: u64 },
    NodesPerSuggest { nodes: u64 },
}

impl SearchBudgetConfig {
    pub fn starts_worker(&self) -> bool {
        matches!(self, SearchBudgetConfig::Background { .. })
    }
}
