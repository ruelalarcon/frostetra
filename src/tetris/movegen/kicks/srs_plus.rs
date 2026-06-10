use crate::tetris::model::Rotation;
use crate::tetris::movegen::kicks::srs::ZERO_180_KICKS;
use crate::tetris::movegen::kicks::table::{KickTable, KickTransition, TransitionKicks};

use Rotation::*;

pub const JLSTZ_PLUS_KICKS: TransitionKicks = &[
    KickTransition {
        from: North,
        to: East,
        kicks: &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    },
    KickTransition {
        from: East,
        to: North,
        kicks: &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    },
    KickTransition {
        from: East,
        to: South,
        kicks: &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    },
    KickTransition {
        from: South,
        to: East,
        kicks: &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    },
    KickTransition {
        from: South,
        to: West,
        kicks: &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    },
    KickTransition {
        from: West,
        to: South,
        kicks: &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    },
    KickTransition {
        from: West,
        to: North,
        kicks: &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    },
    KickTransition {
        from: North,
        to: West,
        kicks: &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    },
    KickTransition {
        from: North,
        to: South,
        kicks: &[(0, 0), (0, 1), (1, 1), (-1, 1), (1, 0), (-1, 0)],
    },
    KickTransition {
        from: East,
        to: West,
        kicks: &[(0, 0), (1, 0), (1, 2), (1, 1), (0, 2), (0, 1)],
    },
    KickTransition {
        from: South,
        to: North,
        kicks: &[(0, 0), (0, -1), (-1, -1), (1, -1), (-1, 0), (1, 0)],
    },
    KickTransition {
        from: West,
        to: East,
        kicks: &[(0, 0), (-1, 0), (-1, 2), (-1, 1), (0, 2), (0, 1)],
    },
];

pub const I_KICKS: TransitionKicks = &[
    KickTransition {
        from: North,
        to: East,
        kicks: &[(1, 0), (2, 0), (-1, 0), (-1, -1), (2, 2)],
    },
    KickTransition {
        from: East,
        to: North,
        kicks: &[(-1, 0), (-2, 0), (1, 0), (-2, -2), (1, 1)],
    },
    KickTransition {
        from: East,
        to: South,
        kicks: &[(0, -1), (-1, -1), (2, -1), (-1, 1), (2, -2)],
    },
    KickTransition {
        from: South,
        to: East,
        kicks: &[(0, 1), (-2, 1), (1, 1), (-2, 2), (1, -1)],
    },
    KickTransition {
        from: South,
        to: West,
        kicks: &[(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
    },
    KickTransition {
        from: West,
        to: South,
        kicks: &[(1, -1), (-1, -1), (2, -1), (-1, 0), (2, 0)],
    },
    KickTransition {
        from: West,
        to: North,
        kicks: &[(0, 1), (1, 1), (-2, 1), (1, -1), (-2, 2)],
    },
    KickTransition {
        from: North,
        to: West,
        kicks: &[(0, -1), (-1, -1), (2, -1), (2, -2), (-1, 1)],
    },
    KickTransition {
        from: North,
        to: South,
        kicks: &[(1, -1), (1, 0)],
    },
    KickTransition {
        from: East,
        to: West,
        kicks: &[(-1, -1), (0, -1)],
    },
    KickTransition {
        from: South,
        to: North,
        kicks: &[(-1, 1), (-1, 0)],
    },
    KickTransition {
        from: West,
        to: East,
        kicks: &[(1, 1), (0, 1)],
    },
];

pub const SRS_PLUS: KickTable = KickTable {
    i: I_KICKS,
    o: ZERO_180_KICKS,
    j: JLSTZ_PLUS_KICKS,
    l: JLSTZ_PLUS_KICKS,
    s: JLSTZ_PLUS_KICKS,
    t: JLSTZ_PLUS_KICKS,
    z: JLSTZ_PLUS_KICKS,
};
