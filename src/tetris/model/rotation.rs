use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rotation {
    North,
    West,
    South,
    East,
}

impl Rotation {
    pub const fn rotate_cell(self, (x, y): (i8, i8)) -> (i8, i8) {
        match self {
            Rotation::North => (x, y),
            Rotation::East => (y, -x),
            Rotation::South => (-x, -y),
            Rotation::West => (-y, x),
        }
    }

    pub const fn rotate_cells(self, cells: [(i8, i8); 4]) -> [(i8, i8); 4] {
        [
            self.rotate_cell(cells[0]),
            self.rotate_cell(cells[1]),
            self.rotate_cell(cells[2]),
            self.rotate_cell(cells[3]),
        ]
    }

    pub const fn cw(self) -> Self {
        match self {
            Rotation::North => Rotation::East,
            Rotation::East => Rotation::South,
            Rotation::South => Rotation::West,
            Rotation::West => Rotation::North,
        }
    }

    pub const fn ccw(self) -> Self {
        match self {
            Rotation::North => Rotation::West,
            Rotation::East => Rotation::North,
            Rotation::South => Rotation::East,
            Rotation::West => Rotation::South,
        }
    }

    pub const fn flip(self) -> Self {
        match self {
            Rotation::North => Rotation::South,
            Rotation::East => Rotation::West,
            Rotation::South => Rotation::North,
            Rotation::West => Rotation::East,
        }
    }
}
