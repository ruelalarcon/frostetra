use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct SearchConfig {
    pub rng: SearchRngConfig,
    pub budget: SearchBudgetConfig,
}

impl SearchConfig {
    pub(super) fn merge(&mut self, override_config: SearchConfigOverride) {
        if let Some(rng) = override_config.rng {
            self.rng = rng;
        }
        if let Some(budget) = override_config.budget {
            self.budget = budget;
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
        }

        let config = RequiredSearchConfig::deserialize(deserializer)?;
        Ok(SearchConfig {
            rng: config.rng,
            budget: config.budget,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SearchConfigOverride {
    rng: Option<SearchRngConfig>,
    budget: Option<SearchBudgetConfig>,
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
