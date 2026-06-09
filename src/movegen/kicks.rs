use serde::Deserialize;

use crate::data::{Piece, Rotation};

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kickset {
    #[default]
    Srs,
}

pub type KickList = &'static [(i8, i8)];

const NO_KICKS: KickList = &[];

impl Kickset {
    pub fn kicks_between(self, piece: Piece, from: Rotation, to: Rotation) -> KickList {
        use Kickset::*;
        use Piece::*;
        use Rotation::*;

        match (self, piece, from, to) {
            (Srs, I, North, East) => &[(1, 0), (-1, 0), (2, 0), (-1, -1), (2, 2)],
            (Srs, I, East, North) => &[(-1, 0), (1, 0), (-2, 0), (1, 1), (-2, -2)],
            (Srs, I, East, South) => &[(0, -1), (-1, -1), (2, -1), (-1, 1), (2, -2)],
            (Srs, I, South, East) => &[(0, 1), (1, 1), (-2, 1), (1, -1), (-2, 2)],
            (Srs, I, South, West) => &[(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
            (Srs, I, West, South) => &[(1, -1), (-1, -1), (2, -1), (-1, 0), (2, 0)],
            (Srs, I, West, North) => &[(0, 1), (1, 1), (-2, 1), (1, -1), (-2, 2)],
            (Srs, I, North, West) => &[(0, -1), (-1, -1), (2, -1), (-1, 1), (2, -2)],

            (Srs, J | L | S | T | Z, North, East) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
            (Srs, J | L | S | T | Z, East, North) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            (Srs, J | L | S | T | Z, East, South) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            (Srs, J | L | S | T | Z, South, East) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
            (Srs, J | L | S | T | Z, South, West) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
            (Srs, J | L | S | T | Z, West, South) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            (Srs, J | L | S | T | Z, West, North) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            (Srs, J | L | S | T | Z, North, West) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],

            (Srs, _, North, South) => &[(0, 0)],
            (Srs, _, East, West) => &[(0, 0)],
            (Srs, _, South, North) => &[(0, 0)],
            (Srs, _, West, East) => &[(0, 0)],

            _ => NO_KICKS,
        }
    }
}
