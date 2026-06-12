use enumset::EnumSet;

use crate::tetris::model::rules::GameRules;
use crate::tetris::model::{Board, Piece, Placement, PlacementInfo, Spin};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameState {
    pub board: Board,
    pub bag: EnumSet<Piece>,
    pub reserve: Piece,
    pub back_to_back: u8,
    pub combo: u8,
}

impl GameState {
    pub fn advance(
        &mut self,
        next: Piece,
        placement: Placement,
        rules: &GameRules,
    ) -> PlacementInfo {
        self.bag.remove(next);
        if self.bag.is_empty() {
            self.bag = EnumSet::all();
        }
        if placement.location.piece != next {
            self.reserve = next;
        }
        self.board.place(placement.location);
        let cleared_mask = self.board.line_clears();
        let mut perfect_clear = false;
        if cleared_mask != 0 {
            self.board.remove_lines(cleared_mask);
            perfect_clear = self.board.cols.iter().all(|&c| c == 0);
            let hard = cleared_mask.count_ones() == 4
                || (!matches!(placement.spin, Spin::None)
                    && (matches!(placement.location.piece, Piece::T) || rules.allspin_b2b))
                || (rules.allclear_b2b && perfect_clear);
            self.back_to_back = if hard {
                self.back_to_back.saturating_add(1)
            } else {
                0
            };
            self.combo = self.combo.saturating_add(1);
        } else {
            self.combo = 0;
        }
        PlacementInfo {
            placement,
            lines_cleared: cleared_mask.count_ones(),
            combo: self.combo as u32,
            back_to_back: self.back_to_back as u32,
            perfect_clear,
        }
    }
}
