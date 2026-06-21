use crate::bot::behavior::freestyle::features::{board, tslot};
use crate::bot::behavior::freestyle::score::{Eval, Reward};
use crate::bot::behavior::freestyle::weights::Weights;
use crate::tetris::model::{BoardRepresentation, GameState, Piece, PlacementInfo, Spin};

pub fn evaluate<B: BoardRepresentation>(
    weights: &Weights,
    mut state: GameState<B>,
    info: &PlacementInfo,
    softdrop: u32,
) -> (Eval, Reward) {
    let mut eval = 0.0;
    let mut reward = 0.0;

    if info.perfect_clear {
        reward += weights.perfect_clear;
    }
    if !info.perfect_clear || !weights.perfect_clear_override {
        if info.lines_cleared != 0 && info.back_to_back > 1 {
            reward += weights.back_to_back_clear;
        }
        reward += match (
            info.placement.location.piece,
            info.placement.spin,
            info.lines_cleared as usize,
        ) {
            (_, Spin::None, lines) => weights.normal_clears[lines],
            (Piece::T, Spin::Mini, lines) => weights.t_spin_mini_clears[lines],
            (Piece::T, Spin::Full, lines) => weights.t_spin_clears[lines],
            (_, Spin::Mini, lines) => weights.allspin_mini_clears[lines],
            (_, Spin::Full, lines) => weights.allspin_clears[lines],
        };
        reward += weights.combo_attack * (info.combo.saturating_sub(1) / 2) as f32;
    }

    if info.placement.location.piece == Piece::T
        && (info.lines_cleared < 2 || !matches!(info.placement.spin, Spin::Full))
    {
        reward += weights.wasted_t;
    }
    if state.back_to_back > 0 {
        eval += weights.has_back_to_back;
    }
    reward += weights.softdrop * softdrop as f32;

    let cutout_count = state.bag.contains(Piece::T) as usize
        + (state.reserve == Piece::T) as usize
        + (state.bag.len() <= 3) as usize;
    for _ in 0..cutout_count {
        let location = match tslot::well_known_tslot(&state.board) {
            Some(v) => v,
            None => break,
        };
        let mut board = state.board.clone();
        board.place(location);
        eval += weights.tslot[board.line_clears().count_ones() as usize];
        if board.line_clears().count_ones() > 1 {
            board.remove_lines(board.line_clears());
            state.board = board;
        }
    }

    eval += weights.holes * board::holes(&state.board) as f32;
    eval += weights.cell_coveredness
        * board::coveredness(&state.board, weights.max_cell_covered_height) as f32;
    eval += board::tetris_well_depth(&state.board) as f32 * weights.tetris_well_depth;

    let highest_point = board::highest_point(&state.board);
    eval += weights.height * highest_point as f32;
    if highest_point > 10 {
        eval += weights.height_upper_half * (highest_point - 10) as f32;
    }
    if highest_point > 15 {
        eval += weights.height_upper_quarter * (highest_point - 15) as f32;
    }

    eval += board::row_transitions(&state.board) as f32 * weights.row_transitions;

    (Eval::new(eval), Reward::new(reward))
}
