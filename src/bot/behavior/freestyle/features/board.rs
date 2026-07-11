use crate::tetris::model::BoardRepresentation;

pub fn tetris_well(board: &impl BoardRepresentation) -> (usize, u32) {
    let mut well_column = 0;
    let mut well_height = height(board.cols()[0]);
    for (x, &col) in board.cols().iter().enumerate().skip(1) {
        let col_height = height(col);
        if col_height <= well_height {
            well_column = x;
            well_height = col_height;
        }
    }
    let full_lines_except_well = board
        .cols()
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != well_column)
        .map(|(_, &c)| c)
        .fold(mask_below(board.height() as u32), |a, b| a & b);
    (
        well_column,
        (full_lines_except_well >> well_height).trailing_ones(),
    )
}

pub fn highest_point(board: &impl BoardRepresentation) -> u32 {
    board
        .cols()
        .iter()
        .map(|&c| 64 - c.leading_zeros())
        .max()
        .unwrap()
}

pub fn bumpiness(board: &impl BoardRepresentation, well: usize) -> (u32, u32) {
    let heights = column_heights(board);
    let mut bumpiness = -1;
    let mut bumpiness_sq = -1;

    let mut prev = if well == 0 { 1 } else { 0 };
    for i in 1..heights.len() {
        if i == well {
            continue;
        }
        let delta = (heights[prev] - heights[i]).abs();
        bumpiness += delta;
        bumpiness_sq += delta * delta;
        prev = i;
    }

    (bumpiness.unsigned_abs(), bumpiness_sq.unsigned_abs())
}

pub fn cavities_and_overhangs(board: &impl BoardRepresentation) -> (u32, u32) {
    let heights = column_heights(board);
    let highest_point = heights.iter().copied().max().unwrap_or(0);
    let mut cavities = 0;
    let mut overhangs = 0;

    for y in 0..highest_point {
        for x in 0..board.width() {
            if board.occupied((x as i8, y as i8)) || y >= heights[x] {
                continue;
            }

            if x > 1 && heights[x - 1] <= y - 1 && heights[x - 2] <= y {
                overhangs += 1;
                continue;
            }

            if x + 2 < board.width() && heights[x + 1] <= y - 1 && heights[x + 2] <= y {
                overhangs += 1;
                continue;
            }

            cavities += 1;
        }
    }

    (cavities, overhangs)
}

pub fn covered_cells(board: &impl BoardRepresentation) -> (u32, u32) {
    let heights = column_heights(board);
    let mut covered = 0;
    let mut covered_sq = 0;

    for (x, height) in heights.iter().copied().enumerate() {
        for y in (0..height.saturating_sub(2)).rev() {
            if !board.occupied((x as i8, y as i8)) {
                let cells = 6.min(height - y - 1);
                covered += cells;
                covered_sq += cells * cells;
            }
        }
    }

    (covered as u32, covered_sq as u32)
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

fn column_heights(board: &impl BoardRepresentation) -> Vec<i32> {
    board.cols().iter().map(|&c| height(c) as i32).collect()
}

fn height(col: u64) -> u32 {
    64 - col.leading_zeros()
}

fn mask_below(height: u32) -> u64 {
    if height >= 64 {
        !0
    } else {
        (1 << height) - 1
    }
}
