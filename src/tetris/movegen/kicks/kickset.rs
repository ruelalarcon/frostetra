use serde::Deserialize;

use crate::tetris::model::{Piece, Rotation};
use crate::tetris::movegen::kicks::table::KickList;
use crate::tetris::movegen::kicks::{srs, srs_plus};

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kickset {
    #[default]
    Srs,
    SrsPlus,
}

impl Kickset {
    pub fn kicks_between(self, piece: Piece, from: Rotation, to: Rotation) -> KickList {
        match self {
            Kickset::Srs => srs::SRS.kicks_between(piece, from, to),
            Kickset::SrsPlus => srs_plus::SRS_PLUS.kicks_between(piece, from, to),
        }
    }
}
