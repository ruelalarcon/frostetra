use crate::tetris::model::{BoardRepresentation, Piece, PieceLocation, Rotation};

pub fn well_known_tslot(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    well_known_tslot_left(board).or_else(|| well_known_tslot_right(board))
}

fn well_known_tslot_left(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    for (x, cols) in board.cols().windows(3).enumerate() {
        let y = 64 - cols[0].leading_zeros();
        if 64 - cols[1].leading_zeros() >= y {
            continue;
        }
        if !board.occupied((x as i8 + 2, y as i8 - 1)) {
            continue;
        }
        if board.occupied((x as i8 + 2, y as i8)) {
            continue;
        }
        if !board.occupied((x as i8 + 2, y as i8 + 1)) {
            continue;
        }
        return Some(PieceLocation {
            piece: Piece::T,
            rotation: Rotation::South,
            x: x as i8 + 1,
            y: y as i8,
        });
    }
    None
}

fn well_known_tslot_right(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    for (x, cols) in board.cols().windows(3).enumerate() {
        let y = 64 - cols[2].leading_zeros();
        if 64 - cols[1].leading_zeros() >= y {
            continue;
        }
        if !board.occupied((x as i8, y as i8 - 1)) {
            continue;
        }
        if board.occupied((x as i8, y as i8)) {
            continue;
        }
        if !board.occupied((x as i8, y as i8 + 1)) {
            continue;
        }
        return Some(PieceLocation {
            piece: Piece::T,
            rotation: Rotation::South,
            x: x as i8 + 1,
            y: y as i8,
        });
    }
    None
}
