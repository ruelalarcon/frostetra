use crate::tetris::model::{BoardRepresentation, Piece, PieceLocation, Rotation};

/// Recognize promising T setups without running move generation during evaluation.
/// These patterns estimate potential; they do not prove a reachable rotation path.
pub fn well_known_tslot(board: &impl BoardRepresentation) -> Option<PieceLocation> {
    well_known_tslot_left(board)
        .filter(|location| !location.obstructed(board))
        .or_else(|| well_known_tslot_right(board))
        .filter(|location| !location.obstructed(board))
        .or_else(|| tst_twist_left(board).and_then(|tst| twist_tslot(board, tst)))
        .or_else(|| tst_twist_right(board).and_then(|tst| twist_tslot(board, tst)))
        .or_else(|| fin_left(board))
        .filter(|location| !location.obstructed(board))
        .or_else(|| fin_right(board))
        .filter(|location| !location.obstructed(board))
}

fn twist_tslot(board: &impl BoardRepresentation, tst: PieceLocation) -> Option<PieceLocation> {
    if tst.obstructed(board) {
        return None;
    }
    cave_tslot(board, tst)
        .filter(|location| !location.obstructed(board))
        .or_else(|| {
            let corners = board.occupied((tst.x - 1, tst.y - 1)) as usize
                + board.occupied((tst.x + 1, tst.y - 1)) as usize
                + board.occupied((tst.x - 1, tst.y + 1)) as usize
                + board.occupied((tst.x + 1, tst.y + 1)) as usize;
            (corners >= 3 && on_stack(board, tst)).then_some(tst)
        })
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
            && board.occupied((x + 1, h1 - 2))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tetris::model::DynamicBoard;

    fn assert_slot_and_mirror(cols: &[u64], expected: PieceLocation, lines: u32) {
        let mut board = DynamicBoard::new(cols.into(), 40);
        assert_eq!(well_known_tslot(&board), Some(expected));
        assert!(!expected.obstructed(&board));
        assert!(on_stack(&board, expected));
        board.place(expected);
        assert_eq!(board.line_clears().count_ones(), lines);

        let mut mirrored = DynamicBoard::new(cols.iter().rev().copied().collect(), 40);
        let expected = PieceLocation {
            x: cols.len() as i8 - 1 - expected.x,
            rotation: match expected.rotation {
                Rotation::East => Rotation::West,
                Rotation::West => Rotation::East,
                rotation => rotation,
            },
            ..expected
        };
        assert_eq!(well_known_tslot(&mirrored), Some(expected));
        assert!(!expected.obstructed(&mirrored));
        assert!(on_stack(&mirrored, expected));
        mirrored.place(expected);
        assert_eq!(mirrored.line_clears().count_ones(), lines);
    }

    #[test]
    fn recognizes_basic_slots() {
        assert_slot_and_mirror(
            &[0b11, 0, 0b1010],
            PieceLocation {
                piece: Piece::T,
                rotation: Rotation::South,
                x: 1,
                y: 2,
            },
            2,
        );
    }

    #[test]
    fn recognizes_triple_twists() {
        assert_slot_and_mirror(
            &[0b1110, 0b1010, 0b100001, 0b1110, 0b1110, 0b1110],
            PieceLocation {
                piece: Piece::T,
                rotation: Rotation::West,
                x: 2,
                y: 2,
            },
            3,
        );
    }

    #[test]
    fn recognizes_cave_slots() {
        assert_slot_and_mirror(
            &[0b1110, 0b1010, 0b100001, 0b1010, 0b1110, 0b1110],
            PieceLocation {
                piece: Piece::T,
                rotation: Rotation::South,
                x: 2,
                y: 2,
            },
            2,
        );
    }

    #[test]
    fn recognizes_fin_slots() {
        assert_slot_and_mirror(
            &[0b1110, 0b110, 0b100010, 0b100000, 0b1110, 0b1110],
            PieceLocation {
                piece: Piece::T,
                rotation: Rotation::West,
                x: 3,
                y: 2,
            },
            2,
        );
    }

    #[test]
    fn ignores_empty_and_flat_boards_of_different_widths() {
        for width in [1, 2, 3, 6, 10, 12] {
            for col in [0, 0b1111] {
                let board = DynamicBoard::new(vec![col; width].into_boxed_slice(), 40);
                assert_eq!(well_known_tslot(&board), None);
            }
        }
    }

    #[test]
    fn rejects_obstructed_twists() {
        let board = DynamicBoard::new(vec![0, 0b1110, 0b100001, 0b10, 0, 0].into(), 40);
        assert_eq!(well_known_tslot(&board), None);
    }

    #[test]
    fn rejects_fins_with_a_missing_corner_on_either_side() {
        let cols = vec![0, 0b100, 0b100000, 0b100000, 0b1010, 0];
        let board = DynamicBoard::new(cols.clone().into(), 40);
        let mirrored = DynamicBoard::new(cols.into_iter().rev().collect(), 40);
        assert_eq!(well_known_tslot(&board), None);
        assert_eq!(well_known_tslot(&mirrored), None);
    }
}
