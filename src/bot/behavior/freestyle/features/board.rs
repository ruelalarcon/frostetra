use crate::tetris::model::BoardRepresentation;

pub fn holes(board: &impl BoardRepresentation) -> u32 {
    board
        .cols()
        .iter()
        .map(|&c| {
            let height = 64 - c.leading_zeros();
            let underneath = mask_below(height);
            let holes = !c & underneath;
            holes.count_ones()
        })
        .sum()
}

pub fn coveredness(board: &impl BoardRepresentation, max_cell_covered_height: u32) -> u32 {
    let mut coveredness = 0;
    for &c in board.cols() {
        let height = 64 - c.leading_zeros();
        let underneath = mask_below(height);
        let mut holes = !c & underneath;
        while holes != 0 {
            let y = holes.trailing_zeros();
            coveredness += (height - y).min(max_cell_covered_height);
            holes &= !(1 << y);
        }
    }
    coveredness
}

pub fn tetris_well_depth(board: &impl BoardRepresentation) -> u32 {
    let (tetris_well_column, tetris_well_height) = board
        .cols()
        .iter()
        .enumerate()
        .map(|(i, &c)| (i, 64 - c.leading_zeros()))
        .min_by_key(|&(_, h)| h)
        .unwrap();
    let full_lines_except_well = board
        .cols()
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != tetris_well_column)
        .map(|(_, &c)| c)
        .fold(mask_below(board.height() as u32), |a, b| a & b);
    (full_lines_except_well >> tetris_well_height).trailing_ones()
}

pub fn highest_point(board: &impl BoardRepresentation) -> u32 {
    board
        .cols()
        .iter()
        .map(|&c| 64 - c.leading_zeros())
        .max()
        .unwrap()
}

pub fn row_transitions(board: &impl BoardRepresentation) -> u32 {
    let mask = mask_below(board.height() as u32);
    let cols = board.cols();
    let mut row_transitions = 0;
    row_transitions = row_transitions + (mask ^ cols[0]).count_ones();
    row_transitions = row_transitions + (mask ^ cols[cols.len() - 1]).count_ones();
    for cs in cols.windows(2) {
        row_transitions += ((cs[0] ^ cs[1]) & mask).count_ones();
    }
    row_transitions
}

fn mask_below(height: u32) -> u64 {
    if height >= 64 {
        !0
    } else {
        (1 << height) - 1
    }
}
