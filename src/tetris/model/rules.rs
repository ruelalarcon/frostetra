use serde::Deserialize;

use crate::tetris::movegen::Kickset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameRules {
    pub kickset: Kickset,
    pub rot180: bool,
    pub sonic_drop: SonicDrop,
    pub allspin_b2b: bool,
    pub allclear_b2b: bool,
}

impl Default for GameRules {
    fn default() -> Self {
        GameRules {
            kickset: Kickset::default(),
            rot180: true,
            sonic_drop: SonicDrop::Only,
            allspin_b2b: false,
            allclear_b2b: false,
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
