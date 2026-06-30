use crate::tetris::model::{Piece, Rotation};

pub type KickOffset = (i8, i8);
pub type KickList = &'static [KickOffset];

pub const NO_KICKS: KickList = &[];

#[derive(Clone, Copy, Debug)]
pub struct KickTransition {
    pub from: Rotation,
    pub to: Rotation,
    pub kicks: KickList,
}

pub type TransitionKicks = &'static [KickTransition];
pub const NO_TRANSITIONS: TransitionKicks = &[];

#[derive(Clone, Copy, Debug)]
pub struct KickTable {
    pub i: TransitionKicks,
    pub o: TransitionKicks,
    pub j: TransitionKicks,
    pub l: TransitionKicks,
    pub s: TransitionKicks,
    pub t: TransitionKicks,
    pub z: TransitionKicks,
}

impl KickTable {
    pub fn kicks_between(self, piece: Piece, from: Rotation, to: Rotation) -> KickList {
        let transitions = match piece {
            Piece::I => self.i,
            Piece::O => self.o,
            Piece::J => self.j,
            Piece::L => self.l,
            Piece::S => self.s,
            Piece::T => self.t,
            Piece::Z => self.z,
        };

        let Some(index) = transition_index(from, to) else {
            return NO_KICKS;
        };
        // Kick tables are stored in transition_index order. Direct indexing
        // avoids a linear scan in movegen's rotation hot path; debug builds
        // verify that table declarations keep this order.
        transitions
            .get(index)
            .map(|transition| {
                debug_assert_eq!(transition.from, from);
                debug_assert_eq!(transition.to, to);
                transition.kicks
            })
            .unwrap_or(NO_KICKS)
    }
}

const fn transition_index(from: Rotation, to: Rotation) -> Option<usize> {
    use Rotation::*;

    match (from, to) {
        (North, East) => Some(0),
        (East, North) => Some(1),
        (East, South) => Some(2),
        (South, East) => Some(3),
        (South, West) => Some(4),
        (West, South) => Some(5),
        (West, North) => Some(6),
        (North, West) => Some(7),
        (North, South) => Some(8),
        (East, West) => Some(9),
        (South, North) => Some(10),
        (West, East) => Some(11),
        _ => None,
    }
}
