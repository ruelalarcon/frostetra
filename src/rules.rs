use serde::Deserialize;

use crate::movegen::Kickset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameRules {
    pub kickset: Kickset,
    pub rot180: bool,
    pub sonic_drop: SonicDrop,
}

impl Default for GameRules {
    fn default() -> Self {
        GameRules {
            kickset: Kickset::default(),
            rot180: true,
            sonic_drop: SonicDrop::Only,
        }
    }
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SonicDrop {
    #[default]
    Only,
    Allow,
}
