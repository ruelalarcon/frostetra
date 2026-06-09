use crate::tetris::model::Rotation;
use crate::tetris::movegen::kicks::table::{KickTable, KickTransition, TransitionKicks};

use Rotation::*;

pub const JLSTZ_KICKS: TransitionKicks = &[
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
    ZERO_180_NORTH_SOUTH,
    ZERO_180_EAST_WEST,
    ZERO_180_SOUTH_NORTH,
    ZERO_180_WEST_EAST,
];

pub const I_KICKS: TransitionKicks = &[
    KickTransition {
        from: North,
        to: East,
        kicks: &[(1, 0), (-1, 0), (2, 0), (-1, -1), (2, 2)],
    },
    KickTransition {
        from: East,
        to: North,
        kicks: &[(-1, 0), (1, 0), (-2, 0), (1, 1), (-2, -2)],
    },
    KickTransition {
        from: East,
        to: South,
        kicks: &[(0, -1), (-1, -1), (2, -1), (-1, 1), (2, -2)],
    },
    KickTransition {
        from: South,
        to: East,
        kicks: &[(0, 1), (1, 1), (-2, 1), (1, -1), (-2, 2)],
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
        kicks: &[(0, -1), (-1, -1), (2, -1), (-1, 1), (2, -2)],
    },
    ZERO_180_NORTH_SOUTH,
    ZERO_180_EAST_WEST,
    ZERO_180_SOUTH_NORTH,
    ZERO_180_WEST_EAST,
];

pub const ZERO_180_KICKS: TransitionKicks = &[
    ZERO_180_NORTH_SOUTH,
    ZERO_180_EAST_WEST,
    ZERO_180_SOUTH_NORTH,
    ZERO_180_WEST_EAST,
];

pub const SRS: KickTable = KickTable {
    i: I_KICKS,
    o: ZERO_180_KICKS,
    j: JLSTZ_KICKS,
    l: JLSTZ_KICKS,
    s: JLSTZ_KICKS,
    t: JLSTZ_KICKS,
    z: JLSTZ_KICKS,
};

const ZERO_180_NORTH_SOUTH: KickTransition = KickTransition {
    from: North,
    to: South,
    kicks: &[(0, 0)],
};

const ZERO_180_EAST_WEST: KickTransition = KickTransition {
    from: East,
    to: West,
    kicks: &[(0, 0)],
};

const ZERO_180_SOUTH_NORTH: KickTransition = KickTransition {
    from: South,
    to: North,
    kicks: &[(0, 0)],
};

const ZERO_180_WEST_EAST: KickTransition = KickTransition {
    from: West,
    to: East,
    kicks: &[(0, 0)],
};
