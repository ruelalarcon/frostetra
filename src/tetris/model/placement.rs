use serde::{Deserialize, Serialize};

use crate::tetris::model::{PieceLocation, Rotation, Spin};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Placement {
    pub location: PieceLocation,
    pub spin: Spin,
}

impl Placement {
    pub(crate) fn sort_key(&self) -> (usize, usize, i8, i8, usize) {
        (
            self.location.piece as usize,
            rotation_key(self.location.rotation),
            self.location.x,
            self.location.y,
            spin_key(self.spin),
        )
    }
}

const fn rotation_key(rotation: Rotation) -> usize {
    match rotation {
        Rotation::North => 0,
        Rotation::East => 1,
        Rotation::South => 2,
        Rotation::West => 3,
    }
}

const fn spin_key(spin: Spin) -> usize {
    match spin {
        Spin::None => 0,
        Spin::Mini => 1,
        Spin::Full => 2,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlacementInfo {
    pub placement: Placement,
    pub lines_cleared: u32,
    pub combo: u32,
    pub back_to_back: u32,
    pub perfect_clear: bool,
}
