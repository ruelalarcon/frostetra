use crate::tetris::model::{Board, Piece, PieceLocation, Rotation};

pub(super) struct CollisionMaps {
    boards: [[u64; 10]; 4],
    height: u8,
}

impl CollisionMaps {
    pub(super) fn new(board: &Board, piece: Piece) -> Self {
        let mut boards = [[0; 10]; 4];
        for rot in [
            Rotation::North,
            Rotation::West,
            Rotation::South,
            Rotation::East,
        ] {
            for (dx, dy) in rot.rotate_cells(piece.cells()) {
                for x in 0..10 {
                    let c = board.cols.get((x + dx) as usize).copied().unwrap_or(!0);
                    let c = match dy < 0 {
                        true => !(!c << -dy),
                        false => c >> dy,
                    };
                    boards[rot as usize][x as usize] |= c;
                }
            }
        }
        CollisionMaps {
            boards,
            height: board.height,
        }
    }

    pub(super) fn obstructed(&self, piece: PieceLocation) -> bool {
        piece.y < 0
            || piece.cells().iter().any(|&(_, y)| y >= self.height as i8)
            || self.boards[piece.rotation as usize]
                .get(piece.x as usize)
                .map(|&c| c & 1 << piece.y != 0)
                .unwrap_or(true)
    }
}
