use std::sync::Arc;

use crate::config::BotConfig;
use crate::tetris::model::rules::GameRules;

#[derive(Debug)]
pub struct BotOptions {
    pub speculate: bool,
    pub rules: GameRules,
    pub config: Arc<BotConfig>,
}
