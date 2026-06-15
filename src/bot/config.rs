use std::sync::Arc;

use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize};

use crate::bot::behavior::freestyle;
use crate::bot::behavior::BehaviorKind;
use crate::tetris::model::rules::GameRules;

#[derive(Serialize, Debug, Clone)]
pub struct BotConfig {
    pub initial_behavior: BehaviorKind,
    pub freestyle: freestyle::FreestyleConfig,
}

impl Default for BotConfig {
    fn default() -> Self {
        static DEFAULT: Lazy<BotConfig> = Lazy::new(|| {
            let config: FullBotConfig =
                serde_json::from_str(include_str!("default_config.json")).unwrap();
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
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct PartialBotConfig {
            initial_behavior: Option<BehaviorKind>,
            freestyle: Option<freestyle::FreestyleConfig>,
        }

        let partial = PartialBotConfig::deserialize(deserializer)?;
        let mut config = BotConfig::default();
        if let Some(initial_behavior) = partial.initial_behavior {
            config.initial_behavior = initial_behavior;
        }
        if let Some(freestyle) = partial.freestyle {
            config.freestyle = freestyle;
        }
        Ok(config)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FullBotConfig {
    initial_behavior: BehaviorKind,
    freestyle: freestyle::FreestyleConfig,
}

impl From<FullBotConfig> for BotConfig {
    fn from(config: FullBotConfig) -> Self {
        BotConfig {
            initial_behavior: config.initial_behavior,
            freestyle: config.freestyle,
        }
    }
}

#[derive(Debug)]
pub struct BotOptions {
    pub speculate: bool,
    pub rules: GameRules,
    pub config: Arc<BotConfig>,
}
