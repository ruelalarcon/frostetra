use serde::Deserialize;

use crate::data::{Piece, Rotation};

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kickset {
    #[default]
    Srs,
}

const fn offsets(piece: Piece, rotation: Rotation) -> [(i8, i8); 5] {
    match piece {
        Piece::O => match rotation {
            Rotation::North => [(0, 0); 5],
            Rotation::East => [(0, -1); 5],
            Rotation::South => [(-1, -1); 5],
            Rotation::West => [(-1, 0); 5],
        },
        Piece::I => match rotation {
            Rotation::North => [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
            Rotation::East => [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
            Rotation::South => [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
            Rotation::West => [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
        },
        _ => match rotation {
            Rotation::North => [(0, 0); 5],
            Rotation::East => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            Rotation::South => [(0, 0); 5],
            Rotation::West => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        },
    }
}

pub const fn kicks(piece: Piece, from: Rotation, to: Rotation) -> [(i8, i8); 5] {
    let mut result = [(0, 0); 5];
    let from = offsets(piece, from);
    let to = offsets(piece, to);
    let mut i = 0;
    while i < result.len() {
        result[i] = (from[i].0 - to[i].0, from[i].1 - to[i].1);
        i += 1;
    }
    result
}

pub const fn kicks_180(_from: Rotation) -> [(i8, i8); 1] {
    [(0, 0)]
}
