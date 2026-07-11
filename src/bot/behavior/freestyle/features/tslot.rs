use crate::tetris::model::{BoardRepresentation, Piece, PieceLocation, Rotation};

pub fn well_known_tslot(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    well_known_tslot_left(board)
        .or_else(|| well_known_tslot_right(board))
        .or_else(|| {
            let tst = tst_twist_left(board).or_else(|| tst_twist_right(board))?;
            cave_tslot(board, tst).or_else(|| {
                let corners = board.occupied((tst.x - 1, tst.y - 1)) as usize
                    + board.occupied((tst.x + 1, tst.y - 1)) as usize
                    + board.occupied((tst.x - 1, tst.y + 1)) as usize
                    + board.occupied((tst.x + 1, tst.y + 1)) as usize;
                if corners >= 3 && on_stack(board, tst) {
                    Some(tst)
                } else {
                    None
                }
            })
        })
        .or_else(|| fin_left(board))
        .or_else(|| fin_right(board))
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

fn tst_twist_left(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    for (x, cols) in board.cols().windows(3).enumerate() {
        let x = x as i8;
        let h1 = height(cols[0]) as i8;
        let h2 = height(cols[1]) as i8;
        if h1 > h2 || board.occupied((x - 1, h2)) != board.occupied((x - 1, h2 + 1)) {
            continue;
        }
        if board.occupied((x + 2, h2 + 1))
            && !board.occupied((x + 2, h2))
            && !board.occupied((x + 2, h2 - 1))
            && !board.occupied((x + 1, h2 - 2))
            && !board.occupied((x + 2, h2 - 2))
        {
            return Some(PieceLocation {
                piece: Piece::T,
                rotation: Rotation::West,
                x: x + 2,
                y: h2 - 2,
            });
        }
    }
    None
}

fn tst_twist_right(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    for (x, cols) in board.cols().windows(3).enumerate() {
        let x = x as i8;
        let h1 = height(cols[1]) as i8;
        let h2 = height(cols[2]) as i8;
        if h2 > h1 || board.occupied((x + 3, h1)) != board.occupied((x + 3, h1 + 1)) {
            continue;
        }
        if board.occupied((x, h1 + 1))
            && !board.occupied((x, h1))
            && !board.occupied((x, h1 - 1))
            && !board.occupied((x, h1 - 2))
            && !board.occupied((x + 1, h1 - 2))
        {
            return Some(PieceLocation {
                piece: Piece::T,
                rotation: Rotation::East,
                x,
                y: h1 - 2,
            });
        }
    }
    None
}

fn fin_left(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    for (x, cols) in board.cols().windows(4).enumerate() {
        let x = x as i8;
        let h1 = height(cols[0]) as i8;
        let h2 = height(cols[1]) as i8;
        if h1 > h2 + 1 {
            continue;
        }
        if board.occupied((x + 2, h2 + 2))
            && board.occupied((x + 3, h2 + 2))
            && !board.occupied((x + 2, h2 + 1))
            && !board.occupied((x + 3, h2 + 1))
            && !board.occupied((x + 2, h2))
            && !board.occupied((x + 3, h2))
            && board.occupied((x + 4, h2))
            && !board.occupied((x + 2, h2 - 1))
            && !board.occupied((x + 3, h2 - 1))
            && board.occupied((x + 2, h2 - 2))
            && !board.occupied((x + 3, h2 - 2))
            && board.occupied((x + 4, h2 - 2))
        {
            return Some(PieceLocation {
                piece: Piece::T,
                rotation: Rotation::West,
                x: x + 3,
                y: h2 - 1,
            });
        }
    }
    None
}

fn fin_right(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    for (x, cols) in board.cols().windows(4).enumerate() {
        let x = x as i8;
        let h1 = height(cols[2]) as i8;
        let h2 = height(cols[3]) as i8;
        if h2 > h1 + 1 || !board.occupied((x - 1, h1)) || !board.occupied((x - 1, h1 - 2)) {
            continue;
        }
        if board.occupied((x, h1 + 2))
            && board.occupied((x + 1, h1 + 2))
            && !board.occupied((x, h1 + 1))
            && !board.occupied((x + 1, h1 + 1))
            && !board.occupied((x, h1))
            && !board.occupied((x + 1, h1))
            && !board.occupied((x, h1 - 1))
            && !board.occupied((x + 1, h1 - 1))
            && !board.occupied((x, h1 - 2))
        {
            return Some(PieceLocation {
                piece: Piece::T,
                rotation: Rotation::East,
                x,
                y: h1 - 1,
            });
        }
    }
    None
}

fn cave_tslot(
    board: &impl BoardRepresentation,
    mut starting_point: PieceLocation,
) -> Option<PieceLocation> {
    starting_point.y -= starting_point.drop_distance(board);
    let x = starting_point.x;
    let y = starting_point.y;
    match starting_point.rotation {
        Rotation::East => {
            if !board.occupied((x - 1, y))
                && board.occupied((x - 1, y - 1))
                && board.occupied((x + 1, y - 1))
                && board.occupied((x - 1, y + 1))
            {
                Some(PieceLocation {
                    piece: Piece::T,
                    rotation: Rotation::South,
                    x,
                    y,
                })
            } else if !board.occupied((x + 1, y - 1))
                && !board.occupied((x + 2, y - 1))
                && !board.occupied((x + 1, y - 2))
                && board.occupied((x - 1, y))
                && board.occupied((x + 2, y))
                && board.occupied((x, y - 2))
                && board.occupied((x + 2, y - 2))
            {
                Some(PieceLocation {
                    piece: Piece::T,
                    rotation: Rotation::South,
                    x: x + 1,
                    y: y - 1,
                })
            } else {
                None
            }
        }
        Rotation::West => {
            if !board.occupied((x + 1, y))
                && board.occupied((x + 1, y + 1))
                && board.occupied((x + 1, y - 1))
                && board.occupied((x - 1, y - 1))
            {
                Some(PieceLocation {
                    piece: Piece::T,
                    rotation: Rotation::South,
                    x,
                    y,
                })
            } else if !board.occupied((x - 1, y - 1))
                && !board.occupied((x - 2, y - 1))
                && !board.occupied((x - 1, y - 2))
                && board.occupied((x + 1, y))
                && board.occupied((x - 2, y))
                && board.occupied((x - 2, y - 2))
                && board.occupied((x, y - 2))
            {
                Some(PieceLocation {
                    piece: Piece::T,
                    rotation: Rotation::South,
                    x: x - 1,
                    y: y - 1,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn on_stack(board: &impl BoardRepresentation, location: PieceLocation) -> bool {
    PieceLocation {
        y: location.y - 1,
        ..location
    }
    .obstructed(board)
}

fn height(col: u64) -> u32 {
    64 - col.leading_zeros()
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
