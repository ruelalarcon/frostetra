use crate::tetris::model::rules::{GameRules, SpinDetection};
use crate::tetris::model::{BoardRepresentation, Piece, PieceLocation, Spin};

pub fn detect_spin(
    location: PieceLocation,
    board: &impl BoardRepresentation,
    rules: &GameRules,
    kick_index: usize,
) -> Spin {
    let detected = match rules.spin_detection {
        SpinDetection::None => None,
        SpinDetection::TSpins => as_option(detect_t_spin(location, board, kick_index, false)),
        SpinDetection::TSpinsPlus => {
            detect_t_spin_or_immobile_t(location, board, kick_index, false)
        }
        SpinDetection::All => as_option(detect_t_spin(location, board, kick_index, false))
            .or_else(|| non_t_spin(location, board, Spin::Full)),
        SpinDetection::AllPlus => detect_t_spin_or_immobile_t(location, board, kick_index, false)
            .or_else(|| non_t_spin(location, board, Spin::Full)),
        SpinDetection::AllMini => as_option(detect_t_spin(location, board, kick_index, false))
            .or_else(|| non_t_spin(location, board, Spin::Mini)),
        SpinDetection::AllMiniPlus => {
            detect_t_spin_or_immobile_t(location, board, kick_index, false)
                .or_else(|| non_t_spin(location, board, Spin::Mini))
        }
        SpinDetection::MiniOnly => detect_t_spin_or_immobile_t(location, board, kick_index, true)
            .or_else(|| non_t_spin(location, board, Spin::Mini)),
    };
    detected.unwrap_or(Spin::None)
}

fn as_option(spin: Spin) -> Option<Spin> {
    if matches!(spin, Spin::None) {
        None
    } else {
        Some(spin)
    }
}

fn detect_t_spin_or_immobile_t(
    location: PieceLocation,
    board: &impl BoardRepresentation,
    kick_index: usize,
    force_mini: bool,
) -> Option<Spin> {
    match detect_t_spin(location, board, kick_index, force_mini) {
        Spin::None if location.piece == Piece::T && immobile(location, board) => Some(Spin::Mini),
        Spin::None => None,
        spin => Some(spin),
    }
}

fn detect_t_spin(
    location: PieceLocation,
    board: &impl BoardRepresentation,
    kick_index: usize,
    force_mini: bool,
) -> Spin {
    if location.piece != Piece::T {
        return Spin::None;
    }
    if !(PieceLocation {
        y: location.y - 1,
        ..location
    })
    .obstructed(board)
    {
        return Spin::None;
    }
    let corners = [(-1, -1), (1, -1), (-1, 1), (1, 1)]
        .iter()
        .filter(|&&(cx, cy)| board.occupied((cx + location.x, cy + location.y)))
        .count();
    if corners < 3 {
        return Spin::None;
    }
    if force_mini {
        return Spin::Mini;
    }
    let front_corners = [(-1, 1), (1, 1)]
        .iter()
        .map(|&c| location.rotation.rotate_cell(c))
        .filter(|&(cx, cy)| board.occupied((cx + location.x, cy + location.y)))
        .count();
    if front_corners == 2 || kick_index == 4 {
        Spin::Full
    } else {
        Spin::Mini
    }
}

fn non_t_spin(
    location: PieceLocation,
    board: &impl BoardRepresentation,
    spin: Spin,
) -> Option<Spin> {
    if location.piece != Piece::T && immobile(location, board) {
        Some(spin)
    } else {
        None
    }
}

fn immobile(location: PieceLocation, board: &impl BoardRepresentation) -> bool {
    [
        PieceLocation {
            x: location.x - 1,
            ..location
        },
        PieceLocation {
            x: location.x + 1,
            ..location
        },
        PieceLocation {
            y: location.y + 1,
            ..location
        },
    ]
    .iter()
    .all(|location| location.obstructed(board))
}
