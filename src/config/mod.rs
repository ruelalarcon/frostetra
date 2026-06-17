mod search;

use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize};

use crate::bot::behavior::freestyle;
use crate::bot::behavior::BehaviorKind;

use search::SearchConfigOverride;

pub use search::{SearchBudgetConfig, SearchConfig, SearchRngConfig};

#[derive(Serialize, Debug, Clone)]
pub struct BotConfig {
    pub behavior: BehaviorSelectionConfig,
    pub search: SearchConfig,
    pub behaviors: BehaviorConfigs,
}

impl Default for BotConfig {
    fn default() -> Self {
        static DEFAULT: Lazy<BotConfig> = Lazy::new(|| {
            let config: RequiredBotConfig =
                serde_json::from_str(include_str!("default.json")).unwrap();
            config.into()
        });
        DEFAULT.clone()
    }
}

impl<'de> Deserialize<'de> for BotConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let override_config = BotConfigOverride::deserialize(deserializer)?;
        let mut config = BotConfig::default();

        if let Some(behavior) = override_config.behavior {
            config.behavior.merge(behavior);
        }
        if let Some(search) = override_config.search {
            config.search.merge(search);
        }
        if let Some(behaviors) = override_config.behaviors {
            config.behaviors.merge(behaviors);
        }

        Ok(config)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequiredBotConfig {
    behavior: BehaviorSelectionConfig,
    search: SearchConfig,
    behaviors: BehaviorConfigs,
}

impl From<RequiredBotConfig> for BotConfig {
    fn from(config: RequiredBotConfig) -> Self {
        BotConfig {
            behavior: config.behavior,
            search: config.search,
            behaviors: config.behaviors,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BotConfigOverride {
    behavior: Option<BehaviorSelectionConfigOverride>,
    search: Option<SearchConfigOverride>,
    behaviors: Option<BehaviorConfigsOverride>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BehaviorSelectionConfig {
    pub initial: BehaviorKind,
}

impl BehaviorSelectionConfig {
    fn merge(&mut self, override_config: BehaviorSelectionConfigOverride) {
        if let Some(initial) = override_config.initial {
            self.initial = initial;
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BehaviorSelectionConfigOverride {
    initial: Option<BehaviorKind>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BehaviorConfigs {
    pub freestyle: freestyle::FreestyleConfig,
}

impl BehaviorConfigs {
    fn merge(&mut self, override_config: BehaviorConfigsOverride) {
        if let Some(freestyle) = override_config.freestyle {
            self.freestyle = freestyle;
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BehaviorConfigsOverride {
    freestyle: Option<freestyle::FreestyleConfig>,
}
