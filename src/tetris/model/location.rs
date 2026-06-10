use serde::{Deserialize, Serialize};

use crate::tetris::model::{Board, Piece, Rotation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PieceLocation {
    #[serde(rename = "piece", alias = "type")]
    pub piece: Piece,
    #[serde(rename = "orientation")]
    pub rotation: Rotation,
    pub x: i8,
    pub y: i8,
}

macro_rules! lutify {
    (($e:expr) for $v:ident in [$($val:expr),*]) => {
        [
            $(
                {
                    let $v = $val;
                    $e
                }
            ),*
        ]
    };
}

macro_rules! piece_lut {
    ($v:ident => $e:expr) => {
        lutify!(($e) for $v in [Piece::I, Piece::O, Piece::T, Piece::L, Piece::J, Piece::S, Piece::Z])
    };
}

macro_rules! rotation_lut {
    ($v:ident => $e:expr) => {
        lutify!(($e) for $v in [Rotation::North, Rotation::West, Rotation::South, Rotation::East])
    };
}

impl PieceLocation {
    pub const fn cells(&self) -> [(i8, i8); 4] {
        const LUT: [[[(i8, i8); 4]; 4]; 7] =
            piece_lut!(piece => rotation_lut!(rotation => rotation.rotate_cells(piece.cells())));
        self.translate_cells(LUT[self.piece as usize][self.rotation as usize])
    }

    const fn translate(&self, (x, y): (i8, i8)) -> (i8, i8) {
        (x + self.x, y + self.y)
    }

    const fn translate_cells(&self, cells: [(i8, i8); 4]) -> [(i8, i8); 4] {
        [
            self.translate(cells[0]),
            self.translate(cells[1]),
            self.translate(cells[2]),
            self.translate(cells[3]),
        ]
    }

    pub fn obstructed(&self, board: &Board) -> bool {
        self.cells().iter().any(|&cell| board.occupied(cell))
    }

    pub fn drop_distance(&self, board: &Board) -> i8 {
        self.cells()
            .iter()
            .map(|&(x, y)| board.distance_to_ground(x, y))
            .min()
            .unwrap()
    }

    pub fn above_stack(&self, board: &Board) -> bool {
        self.cells()
            .iter()
            .all(|&(x, y)| y >= 64 - board.cols[x as usize].leading_zeros() as i8)
    }

    pub fn canonical_form(&self) -> PieceLocation {
        match self.piece {
            Piece::T | Piece::J | Piece::L => *self,
            Piece::O => match self.rotation {
                Rotation::North => *self,
                Rotation::East => PieceLocation {
                    rotation: Rotation::North,
                    y: self.y - 1,
                    ..*self
                },
                Rotation::South => PieceLocation {
                    rotation: Rotation::North,
                    x: self.x - 1,
                    y: self.y - 1,
                    ..*self
                },
                Rotation::West => PieceLocation {
                    rotation: Rotation::North,
                    x: self.x - 1,
                    ..*self
                },
            },
            Piece::S | Piece::Z => match self.rotation {
                Rotation::North | Rotation::East => *self,
                Rotation::South => PieceLocation {
                    rotation: Rotation::North,
                    y: self.y - 1,
                    ..*self
                },
                Rotation::West => PieceLocation {
                    rotation: Rotation::East,
                    x: self.x - 1,
                    ..*self
                },
            },
            Piece::I => match self.rotation {
                Rotation::North | Rotation::East => *self,
                Rotation::South => PieceLocation {
                    rotation: Rotation::North,
                    x: self.x - 1,
                    ..*self
                },
                Rotation::West => PieceLocation {
                    rotation: Rotation::East,
                    y: self.y + 1,
                    ..*self
                },
            },
        }
    }
}
