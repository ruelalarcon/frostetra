use serde::Deserialize;

use crate::tetris::model::{Piece, Rotation};
use crate::tetris::movegen::kicks::srs;
use crate::tetris::movegen::kicks::table::KickList;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kickset {
    #[default]
    Srs,
}

impl Kickset {
    pub fn kicks_between(self, piece: Piece, from: Rotation, to: Rotation) -> KickList {
        match self {
            Kickset::Srs => srs::SRS.kicks_between(piece, from, to),
        }
    }
}
