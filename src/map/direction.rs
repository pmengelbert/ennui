use crate::map::direction::MapDir::South;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum MapDir {
    North,
    South,
    East,
    West,
    Up,
    Down,
    NorthEast,
    // etc
}

impl Default for MapDir {
    fn default() -> Self {
        South
    }
}

impl MapDir {
    const ALL_DIRS: [MapDir; 7] = [
        MapDir::North,
        MapDir::South,
        MapDir::East,
        MapDir::West,
        MapDir::Up,
        MapDir::Down,
        MapDir::NorthEast,
    ];

    pub fn all() -> &'static [Self] {
        &Self::ALL_DIRS
    }
}

impl std::fmt::Debug for MapDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use MapDir::*;
        write!(
            f,
            "{}",
            match self {
                North => "north",
                South => "south",
                East => "east",
                West => "west",
                Up => "up",
                Down => "down",
                NorthEast => "northeast",
            }
        )
    }
}

impl Display for MapDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use MapDir::*;

        write!(
            f,
            "{}",
            match self {
                North => "n",
                South => "s",
                East => "e",
                West => "w",
                Up => "u",
                Down => "d",
                NorthEast => "ne",
            }
        )
    }
}
