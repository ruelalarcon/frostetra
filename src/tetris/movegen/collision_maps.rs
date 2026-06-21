use crate::tetris::model::{
    Board, BoardRepresentation, DynamicBoard, FixedBoard, Piece, PieceLocation, Rotation,
};

pub trait CollisionMap {
    fn obstructed(&self, piece: PieceLocation) -> bool;
}

pub trait MovegenBoard: BoardRepresentation {
    type Maps: CollisionMap;

    fn collision_maps(&self, piece: Piece) -> Self::Maps;
}

pub struct FixedCollisionMaps<const W: usize> {
    boards: [[u64; W]; 4],
    height: u8,
}

impl<const W: usize> FixedCollisionMaps<W> {
    fn new(board: &FixedBoard<W>, piece: Piece) -> Self {
        let mut boards = [[0; W]; 4];
        for rot in [
            Rotation::North,
            Rotation::West,
            Rotation::South,
            Rotation::East,
        ] {
            for (dx, dy) in rot.rotate_cells(piece.cells()) {
                for x in 0..W {
                    let c = match board.cols().get((x as i8 + dx) as usize).copied() {
                        Some(c) if dy < 0 => !(!c << -dy),
                        Some(c) => c >> dy,
                        None => !0,
                    };
                    boards[rot as usize][x as usize] |= c;
                }
            }
        }
        FixedCollisionMaps {
            boards,
            height: board.height(),
        }
    }
}

impl<const W: usize> CollisionMap for FixedCollisionMaps<W> {
    fn obstructed(&self, piece: PieceLocation) -> bool {
        piece.y < 0
            || piece.cells().iter().any(|&(_, y)| y >= self.height as i8)
            || self.boards[piece.rotation as usize]
                .get(piece.x as usize)
                .map(|&c| c & 1 << piece.y != 0)
                .unwrap_or(true)
    }
}

impl MovegenBoard for Board {
    type Maps = FixedCollisionMaps<10>;

    fn collision_maps(&self, piece: Piece) -> Self::Maps {
        FixedCollisionMaps::new(self, piece)
    }
}

pub struct DynamicCollisionMaps {
    boards: Box<[u64]>,
    width: usize,
    height: u8,
}

impl DynamicCollisionMaps {
    fn new(board: &DynamicBoard, piece: Piece) -> Self {
        let width = board.width();
        let mut boards = vec![0; width * 4].into_boxed_slice();
        for rot in [
            Rotation::North,
            Rotation::West,
            Rotation::South,
            Rotation::East,
        ] {
            let offset = rot as usize * width;
            for (dx, dy) in rot.rotate_cells(piece.cells()) {
                for x in 0..width {
                    let c = match board.cols().get((x as i8 + dx) as usize).copied() {
                        Some(c) if dy < 0 => !(!c << -dy),
                        Some(c) => c >> dy,
                        None => !0,
                    };
                    boards[offset + x] |= c;
                }
            }
        }
        DynamicCollisionMaps {
            boards,
            width,
            height: board.height(),
        }
    }
}

impl CollisionMap for DynamicCollisionMaps {
    fn obstructed(&self, piece: PieceLocation) -> bool {
        piece.y < 0
            || piece.cells().iter().any(|&(_, y)| y >= self.height as i8)
            || (piece.x < 0)
            || self
                .boards
                .get(piece.rotation as usize * self.width + piece.x as usize)
                .map(|&c| c & 1 << piece.y != 0)
                .unwrap_or(true)
    }
}

impl MovegenBoard for DynamicBoard {
    type Maps = DynamicCollisionMaps;

    fn collision_maps(&self, piece: Piece) -> Self::Maps {
        DynamicCollisionMaps::new(self, piece)
    }
}
