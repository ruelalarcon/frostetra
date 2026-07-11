use crate::tetris::model::{Piece, PlacementInfo, Spin};

pub struct TetrioS2AttackCalculator;

impl TetrioS2AttackCalculator {
    const PERFECT_CLEAR_BONUS: u32 = 5;
    const BACK_TO_BACK_BONUS: u32 = 1;
    const BACK_TO_BACK_SURGE_AT: u32 = 4;
    const BACK_TO_BACK_SURGE_BASE: u32 = 3;

    pub fn calculate(&self, info: &PlacementInfo) -> u32 {
        if info.lines_cleared == 0 {
            return 0;
        }

        let combo = normalized_combo(info);
        let back_to_back = normalized_back_to_back(info);
        let back_to_back_before = normalized_back_to_back_before(info);

        let line_clear = t_spin_only_line_clear_attack(info);
        let back_to_back = if back_to_back > 0 {
            Self::BACK_TO_BACK_BONUS
        } else {
            0
        };
        let special_bonus =
            if info.perfect_clear && (info.lines_cleared == 4 || is_t_piece_spin_clear(info)) {
                1
            } else {
                0
            };
        let surge = self.back_to_back_surge(info, back_to_back_before);

        let subtotal = line_clear + back_to_back + special_bonus;
        let line_attack = multiplier_combo_attack(subtotal as f64, combo).floor() as u32;
        let perfect_clear = if info.perfect_clear {
            Self::PERFECT_CLEAR_BONUS
        } else {
            0
        };
        line_attack + perfect_clear + surge
    }

    fn back_to_back_surge(&self, info: &PlacementInfo, back_to_back_before: u32) -> u32 {
        if info.lines_cleared > 0
            && info.back_to_back == 0
            && back_to_back_before > Self::BACK_TO_BACK_SURGE_AT
        {
            back_to_back_before - Self::BACK_TO_BACK_SURGE_AT + Self::BACK_TO_BACK_SURGE_BASE
        } else {
            0
        }
    }
}

fn normalized_combo(info: &PlacementInfo) -> u32 {
    info.combo.saturating_sub(1)
}

fn normalized_back_to_back(info: &PlacementInfo) -> u32 {
    info.back_to_back.saturating_sub(1)
}

fn normalized_back_to_back_before(info: &PlacementInfo) -> u32 {
    info.back_to_back_before.saturating_sub(1)
}

fn multiplier_combo_attack(mut attack: f64, combo: u32) -> f64 {
    if combo == 0 {
        return attack;
    }
    attack *= 1.0 + 0.25 * combo as f64;
    if combo > 1 {
        attack = attack.max((combo as f64 * 1.25).ln_1p());
    }
    attack
}

fn t_spin_only_line_clear_attack(info: &PlacementInfo) -> u32 {
    let lines = info.lines_cleared;
    if info.placement.location.piece != Piece::T {
        return base_line_clear_attack(lines);
    }
    match info.placement.spin {
        Spin::Full => t_spin_attack(lines),
        Spin::Mini => t_spin_mini_attack(lines),
        Spin::None => base_line_clear_attack(lines),
    }
}

fn base_line_clear_attack(lines: u32) -> u32 {
    match lines {
        2 => 1,
        3 => 2,
        4 => 4,
        _ => 0,
    }
}

fn t_spin_attack(lines: u32) -> u32 {
    match lines {
        1 => 2,
        2 => 4,
        3 => 6,
        4 => 10,
        5 => 12,
        _ => 0,
    }
}

fn t_spin_mini_attack(lines: u32) -> u32 {
    match lines {
        2 => 1,
        3 => 2,
        4 => 4,
        _ => 0,
    }
}

fn is_t_piece_spin_clear(info: &PlacementInfo) -> bool {
    info.lines_cleared > 0
        && info.placement.location.piece == Piece::T
        && info.placement.spin == Spin::Full
}

#[cfg(test)]
mod tests {
    use crate::tetris::model::{PieceLocation, Placement, Rotation};

    use super::*;

    #[test]
    fn calculates_base_line_clear_attack() {
        let info = info(Piece::I, Spin::None, 2, 1, 0, false);

        assert_eq!(TetrioS2AttackCalculator.calculate(&info), 1);
    }

    #[test]
    fn applies_back_to_back_to_t_spin_clears() {
        let info = info(Piece::T, Spin::Full, 2, 1, 2, false);

        assert_eq!(TetrioS2AttackCalculator.calculate(&info), 5);
    }

    #[test]
    fn applies_perfect_clear_bonus_and_special_bonus() {
        let info = info(Piece::I, Spin::None, 4, 1, 0, true);

        assert_eq!(TetrioS2AttackCalculator.calculate(&info), 10);
    }

    #[test]
    fn applies_back_to_back_surge_when_breaking_a_large_chain() {
        let mut info = info(Piece::I, Spin::None, 1, 1, 0, false);
        info.back_to_back_before = 6;

        assert_eq!(TetrioS2AttackCalculator.calculate(&info), 4);
    }

    fn info(
        piece: Piece,
        spin: Spin,
        lines_cleared: u32,
        combo: u32,
        back_to_back: u32,
        perfect_clear: bool,
    ) -> PlacementInfo {
        PlacementInfo {
            placement: Placement {
                location: PieceLocation {
                    piece,
                    rotation: Rotation::North,
                    x: 4,
                    y: 20,
                },
                spin,
            },
            lines_cleared,
            combo,
            back_to_back_before: back_to_back.saturating_sub(1),
            back_to_back,
            perfect_clear,
        }
    }
}
