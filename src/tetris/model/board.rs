use serde::Deserialize;

use crate::tetris::model::PieceLocation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(from = "Vec<[Option<char>; 10]>")]
pub struct Board {
    pub cols: [u64; 10],
    pub height: u8,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            cols: [0; 10],
            height: 40,
        }
    }
}

impl Board {
    pub const fn occupied(&self, (x, y): (i8, i8)) -> bool {
        if x < 0 || x >= 10 || y < 0 || y >= self.height as i8 {
            return true;
        }
        self.cols[x as usize] & 1 << y != 0
    }

    pub fn distance_to_ground(&self, x: i8, y: i8) -> i8 {
        debug_assert!((0..10).contains(&x));
        debug_assert!(y >= 0 && y < self.height as i8);
        if y == 0 {
            return 0;
        }
        (!self.cols[x as usize] << (64 - y)).leading_ones() as i8
    }

    pub fn place(&mut self, piece: PieceLocation) {
        for &(x, y) in &piece.cells() {
            debug_assert!((0..10).contains(&x));
            debug_assert!(y >= 0 && y < self.height as i8);
            self.cols[x as usize] |= 1 << y;
        }
    }

    pub fn line_clears(&self) -> u64 {
        self.cols.iter().fold(!0, |a, b| a & b)
    }

    pub fn remove_lines(&mut self, lines: u64) {
        for c in &mut self.cols {
            clear_lines(c, lines);
        }
    }
}

impl From<Vec<[Option<char>; 10]>> for Board {
    fn from(v: Vec<[Option<char>; 10]>) -> Self {
        let mut cols = [0; 10];
        let height = if v.is_empty() { 40 } else { v.len().min(64) };
        for x in 0..10 {
            for y in 0..height {
                if v[y][x].is_some() {
                    cols[x] |= 1 << y;
                }
            }
        }
        Board {
            cols,
            height: height as u8,
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
fn clear_lines(col: &mut u64, lines: u64) {
    *col = unsafe {
        // SAFETY: #[cfg()] guard ensures that this instruction exists at compile time.
        std::arch::x86_64::_pext_u64(*col, !lines)
    };
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
fn clear_lines(col: &mut u64, mut lines: u64) {
    while lines != 0 {
        let i = lines.trailing_zeros();
        let mask = (1 << i) - 1;
        *col = *col & mask | *col >> 1 & !mask;
        lines &= !(1 << i);
        lines >>= 1;
    }
}
