use std::hash::Hash;

use serde::Deserialize;

use crate::tetris::model::PieceLocation;

pub type Board = FixedBoard<10>;

pub trait BoardRepresentation: Clone + Eq + Hash + Send + Sync + 'static {
    fn cols(&self) -> &[u64];
    fn cols_mut(&mut self) -> &mut [u64];
    fn height(&self) -> u8;

    fn width(&self) -> usize {
        self.cols().len()
    }

    fn occupied(&self, (x, y): (i8, i8)) -> bool {
        if x < 0 || y < 0 || y >= self.height() as i8 {
            return true;
        }
        self.cols()
            .get(x as usize)
            .map(|col| col & (1 << y) != 0)
            .unwrap_or(true)
    }

    fn distance_to_ground(&self, x: i8, y: i8) -> i8 {
        debug_assert!(x >= 0 && (x as usize) < self.width());
        debug_assert!(y >= 0 && y < self.height() as i8);
        if y == 0 {
            return 0;
        }
        (!self.cols()[x as usize] << (64 - y)).leading_ones() as i8
    }

    fn place(&mut self, piece: PieceLocation) {
        for &(x, y) in &piece.cells() {
            debug_assert!(x >= 0 && (x as usize) < self.width());
            debug_assert!(y >= 0 && y < self.height() as i8);
            self.cols_mut()[x as usize] |= 1 << y;
        }
    }

    fn line_clears(&self) -> u64 {
        self.cols().iter().fold(!0, |a, b| a & b)
    }

    fn remove_lines(&mut self, lines: u64) {
        for c in self.cols_mut() {
            clear_lines(c, lines);
        }
    }

    fn is_empty(&self) -> bool {
        self.cols().iter().all(|&c| c == 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FixedBoard<const W: usize> {
    pub cols: [u64; W],
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

impl<const W: usize> BoardRepresentation for FixedBoard<W> {
    fn cols(&self) -> &[u64] {
        &self.cols
    }

    fn cols_mut(&mut self) -> &mut [u64] {
        &mut self.cols
    }

    fn height(&self) -> u8 {
        self.height
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DynamicBoard {
    cols: Box<[u64]>,
    height: u8,
}

impl DynamicBoard {
    pub fn new(cols: Box<[u64]>, height: u8) -> Self {
        Self { cols, height }
    }
}

impl BoardRepresentation for DynamicBoard {
    fn cols(&self) -> &[u64] {
        &self.cols
    }

    fn cols_mut(&mut self) -> &mut [u64] {
        &mut self.cols
    }

    fn height(&self) -> u8 {
        self.height
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "Vec<Vec<Option<char>>>")]
pub struct BoardSnapshot {
    cols: Vec<u64>,
    height: u8,
}

impl BoardSnapshot {
    pub fn width(&self) -> usize {
        self.cols.len()
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn into_fixed<const W: usize>(self) -> Result<FixedBoard<W>, BoardSizeError> {
        if self.cols.len() != W {
            return Err(BoardSizeError {
                expected_width: W,
                actual_width: self.cols.len(),
            });
        }

        let mut cols = [0; W];
        cols.copy_from_slice(&self.cols);
        Ok(FixedBoard {
            cols,
            height: self.height,
        })
    }

    pub fn into_dynamic(self) -> DynamicBoard {
        DynamicBoard::new(self.cols.into_boxed_slice(), self.height)
    }

    pub fn into_dynamic_width(self, width: usize) -> Result<DynamicBoard, BoardSizeError> {
        if self.cols.len() != width {
            return Err(BoardSizeError {
                expected_width: width,
                actual_width: self.cols.len(),
            });
        }
        Ok(self.into_dynamic())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardSizeError {
    pub expected_width: usize,
    pub actual_width: usize,
}

impl From<Vec<Vec<Option<char>>>> for BoardSnapshot {
    fn from(rows: Vec<Vec<Option<char>>>) -> Self {
        let height = if rows.is_empty() {
            40
        } else {
            rows.len().min(64)
        };
        let width = rows.iter().map(Vec::len).max().unwrap_or(10);
        let mut cols = vec![0; width];

        for (y, row) in rows.iter().take(height).enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if cell.is_some() {
                    cols[x] |= 1 << y;
                }
            }
        }

        BoardSnapshot {
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
