use crate::tetris::model::{
    Board, BoardRepresentation, DynamicBoard, FixedBoard, Piece, PieceLocation, Rotation,
};

pub trait CollisionMap {
    fn obstructed(&self, piece: PieceLocation) -> bool;
    fn drop_distance(&self, piece: PieceLocation) -> i8;
    fn above_stack(&self, piece: PieceLocation) -> bool;
}

pub trait MovegenBoard: BoardRepresentation {
    type Maps: CollisionMap;

    fn collision_maps(&self, piece: Piece) -> Self::Maps;
}

pub struct FixedCollisionMaps<const W: usize> {
    boards: [[u64; W]; 4],
    above_stack_y: [[i8; W]; 4],
    ceiling_y: [i8; 4],
}

impl<const W: usize> FixedCollisionMaps<W> {
    fn new(board: &FixedBoard<W>, piece: Piece) -> Self {
        let mut boards = [[0; W]; 4];
        let mut above_stack_y = [[0; W]; 4];
        let mut ceiling_y = [0; 4];
        for rot in [
            Rotation::North,
            Rotation::West,
            Rotation::South,
            Rotation::East,
        ] {
            let cells = rot.rotate_cells(piece.cells());
            let max_dy = cells.iter().map(|&(_, y)| y).max().unwrap();
            ceiling_y[rot as usize] = board.height() as i8 - max_dy;
            for x in 0..W {
                above_stack_y[rot as usize][x] = above_stack_y_at(board.cols(), x as i8, cells);
            }
            for (dx, dy) in cells {
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
            above_stack_y,
            ceiling_y,
        }
    }
}

impl<const W: usize> CollisionMap for FixedCollisionMaps<W> {
    fn obstructed(&self, piece: PieceLocation) -> bool {
        piece.y < 0
            || piece.y >= self.ceiling_y[piece.rotation as usize]
            || self.boards[piece.rotation as usize]
                .get(piece.x as usize)
                .map(|&c| c & 1 << piece.y != 0)
                .unwrap_or(true)
    }

    fn drop_distance(&self, piece: PieceLocation) -> i8 {
        drop_distance_to(
            self.boards[piece.rotation as usize][piece.x as usize],
            piece.y,
        )
    }

    fn above_stack(&self, piece: PieceLocation) -> bool {
        piece.y >= self.above_stack_y[piece.rotation as usize][piece.x as usize]
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
    above_stack_y: Box<[i8]>,
    width: usize,
    ceiling_y: [i8; 4],
}

impl DynamicCollisionMaps {
    fn new(board: &DynamicBoard, piece: Piece) -> Self {
        let width = board.width();
        let mut boards = vec![0; width * 4].into_boxed_slice();
        let mut above_stack_y = vec![0; width * 4].into_boxed_slice();
        let mut ceiling_y = [0; 4];
        for rot in [
            Rotation::North,
            Rotation::West,
            Rotation::South,
            Rotation::East,
        ] {
            let offset = rot as usize * width;
            let cells = rot.rotate_cells(piece.cells());
            let max_dy = cells.iter().map(|&(_, y)| y).max().unwrap();
            ceiling_y[rot as usize] = board.height() as i8 - max_dy;
            for x in 0..width {
                above_stack_y[offset + x] = above_stack_y_at(board.cols(), x as i8, cells);
            }
            for (dx, dy) in cells {
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
            above_stack_y,
            width,
            ceiling_y,
        }
    }
}

impl CollisionMap for DynamicCollisionMaps {
    fn obstructed(&self, piece: PieceLocation) -> bool {
        piece.y < 0
            || piece.y >= self.ceiling_y[piece.rotation as usize]
            || (piece.x < 0)
            || self
                .boards
                .get(piece.rotation as usize * self.width + piece.x as usize)
                .map(|&c| c & 1 << piece.y != 0)
                .unwrap_or(true)
    }

    fn drop_distance(&self, piece: PieceLocation) -> i8 {
        let index = piece.rotation as usize * self.width + piece.x as usize;
        drop_distance_to(self.boards[index], piece.y)
    }

    fn above_stack(&self, piece: PieceLocation) -> bool {
        let index = piece.rotation as usize * self.width + piece.x as usize;
        piece.y >= self.above_stack_y[index]
    }
}

fn drop_distance_to(obstructions: u64, y: i8) -> i8 {
    if y == 0 {
        return 0;
    }
    let below = obstructions & ((1 << y) - 1);
    if below == 0 {
        y
    } else {
        y - (63 - below.leading_zeros() as i8) - 1
    }
}

fn above_stack_y_at(cols: &[u64], x: i8, cells: [(i8, i8); 4]) -> i8 {
    cells
        .into_iter()
        .map(|(dx, dy)| {
            let x = x + dx;
            if x < 0 {
                return i8::MAX;
            }
            cols.get(x as usize)
                .map(|&col| column_height(col) - dy)
                .unwrap_or(i8::MAX)
        })
        .max()
        .unwrap()
}

fn column_height(col: u64) -> i8 {
    if col == 0 {
        0
    } else {
        64 - col.leading_zeros() as i8
    }
}

impl MovegenBoard for DynamicBoard {
    type Maps = DynamicCollisionMaps;

    fn collision_maps(&self, piece: Piece) -> Self::Maps {
        DynamicCollisionMaps::new(self, piece)
    }
}
