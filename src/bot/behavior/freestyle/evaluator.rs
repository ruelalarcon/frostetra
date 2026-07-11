use crate::bot::behavior::freestyle::features::{board, tslot};
use crate::bot::behavior::freestyle::score::{Eval, Reward};
use crate::bot::behavior::freestyle::weights::Weights;
use crate::tetris::attack::tetrio_s2::TetrioS2AttackCalculator;
use crate::tetris::model::{BoardRepresentation, GameState, Piece, PlacementInfo, Spin};

pub fn evaluate<B: BoardRepresentation>(
    weights: &Weights,
    state: GameState<B>,
    info: &PlacementInfo,
    move_time: u32,
) -> (Eval, Reward) {
    let highest_point = board::highest_point(&state.board);
    let mut reward = 0.0;

    if info.perfect_clear {
        reward += weights.perfect_clear;
    }
    if weights.stack_pc_damage || !info.perfect_clear {
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
            (_, Spin::Mini, lines) => weights.allspin_clears[lines],
            (_, Spin::Full, lines) => weights.allspin_clears[lines],
        };
        if info.lines_cleared != 0 {
            reward += weights.combo_garbage * combo_garbage(info.combo.saturating_sub(1)) as f32;
        }
    }

    if info.placement.location.piece == Piece::T
        && (info.lines_cleared < 2 || !matches!(info.placement.spin, Spin::Full))
    {
        reward += weights.wasted_t;
    }
    let move_time = move_time + if info.lines_cleared == 0 { 0 } else { 40 };
    reward += weights.move_time * move_time as f32;
    reward += weights.jeopardy
        * highest_point.saturating_sub(10) as f32
        * if weights.timed_jeopardy {
            move_time as f32
        } else {
            10.0
        }
        / 10.0;

    let eval = evaluate_state(weights, state);

    let attack = TetrioS2AttackCalculator.calculate(info);
    (
        Eval::new(eval),
        Reward::new(reward, (info.lines_cleared > 0).then_some(attack)),
    )
}

fn evaluate_state<B: BoardRepresentation>(weights: &Weights, state: GameState<B>) -> f32 {
    let mut state = state;
    let mut eval = 0.0;

    if state.back_to_back > 0 {
        eval += weights.has_back_to_back;
        eval += weights.back_to_back_depth * state.back_to_back as f32;
    }

    let cutout_count = if weights.use_bag {
        state.bag.contains(Piece::T) as usize
            + (state.reserve == Piece::T) as usize
            + (state.bag.len() <= 3) as usize
    } else {
        1 + (state.reserve == Piece::T) as usize
    };
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

    let highest_point = board::highest_point(&state.board);
    eval += weights.height * highest_point as f32;
    if highest_point > 10 {
        eval += weights.top_half * (highest_point - 10) as f32;
    }
    if highest_point > 15 {
        eval += weights.top_quarter * (highest_point - 15) as f32;
    }

    let (well, depth) = board::tetris_well(&state.board);
    let depth = depth.min(weights.max_well_depth);
    eval += weights.well_depth * depth as f32;
    if depth != 0 {
        eval += weights.well_column[well];
    }

    let (bumpiness, bumpiness_sq) = board::bumpiness(&state.board, well);
    eval += weights.bumpiness * bumpiness as f32;
    eval += weights.bumpiness_sq * bumpiness_sq as f32;

    let (cavity_cells, overhang_cells) = board::cavities_and_overhangs(&state.board);
    eval += weights.cavity_cells * cavity_cells as f32;
    eval += weights.cavity_cells_sq * (cavity_cells * cavity_cells) as f32;
    eval += weights.overhang_cells * overhang_cells as f32;
    eval += weights.overhang_cells_sq * (overhang_cells * overhang_cells) as f32;

    let (covered_cells, covered_cells_sq) = board::covered_cells(&state.board);
    eval += weights.covered_cells * covered_cells as f32;
    eval += weights.covered_cells_sq * covered_cells_sq as f32;

    eval += weights.row_transitions * board::row_transitions(&state.board) as f32;

    eval
}

fn combo_garbage(combo: u32) -> u32 {
    const COMBO_GARBAGE: [u32; 12] = [
        0, 0, // 0, 1 combo
        1, 1, // 2, 3 combo
        2, 2, // 4, 5 combo
        3, 3, // 6, 7 combo
        4, 4, 4, // 8, 9, 10 combo
        5, // 11+ combo
    ];
    COMBO_GARBAGE
        .get(combo as usize)
        .copied()
        .unwrap_or(*COMBO_GARBAGE.last().unwrap())
}
