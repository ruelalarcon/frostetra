use serde::{Deserialize, Serialize};

use crate::tetris::model::{PieceLocation, Spin};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Placement {
    pub location: PieceLocation,
    pub spin: Spin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlacementInfo {
    pub placement: Placement,
    pub lines_cleared: u32,
    pub combo: u32,
    pub back_to_back: u32,
    pub perfect_clear: bool,
}
