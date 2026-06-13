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

        transitions
            .iter()
            .find_map(|transition| {
                (transition.from == from && transition.to == to).then_some(transition.kicks)
            })
            .unwrap_or(NO_KICKS)
    }
}
