use std::sync::Arc;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::bot::behavior::freestyle;
use crate::tetris::model::rules::GameRules;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotConfig {
    pub weights: freestyle::Weights,
    pub freestyle_exploitation: f64,
}

impl Default for BotConfig {
    fn default() -> Self {
        static DEFAULT: Lazy<BotConfig> = Lazy::new(|| {
            serde_json::from_str(include_str!("behavior/freestyle/default_weights.json")).unwrap()
        });
        DEFAULT.clone()
    }
}

#[derive(Debug)]
pub struct BotOptions {
    pub speculate: bool,
    pub rules: GameRules,
    pub config: Arc<BotConfig>,
}
