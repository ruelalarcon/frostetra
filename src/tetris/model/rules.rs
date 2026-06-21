use enumset::{enum_set, EnumSet, EnumSetType};
use serde::Deserialize;

use crate::tetris::movegen::Kickset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameRules {
    pub kickset: Kickset,
    pub rot180: bool,
    pub sonic_drop: SonicDrop,
    pub spin_detection: SpinDetection,
    pub back_to_back_sources: EnumSet<BackToBackSource>,
    pub spawn_x: i8,
    pub spawn_y: i8,
}

impl Default for GameRules {
    fn default() -> Self {
        GameRules {
            kickset: Kickset::default(),
            rot180: true,
            sonic_drop: SonicDrop::Only,
            spin_detection: SpinDetection::TSpins,
            back_to_back_sources: enum_set!(
                BackToBackSource::Quad | BackToBackSource::TSpin | BackToBackSource::TSpinMini
            ),
            spawn_x: 4,
            spawn_y: 19,
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

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SpinDetection {
    None,
    #[default]
    TSpins,
    #[serde(rename = "t-spins+")]
    TSpinsPlus,
    All,
    #[serde(rename = "all+")]
    AllPlus,
    AllMini,
    #[serde(rename = "all-mini+")]
    AllMiniPlus,
    MiniOnly,
}

#[derive(EnumSetType, Debug, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackToBackSource {
    Quad,
    TSpin,
    TSpinMini,
    Allspin,
    AllspinMini,
    PerfectClear,
}
