use crate::map::direction::MapDir::{NoneFound, South};
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
    NoneFound,
    // etc
}

impl From<&str> for MapDir {
    fn from(d: &str) -> Self {
        let sw = |a: &str, b: &str| a.starts_with(b);
        match d {
            s if sw(MapDir::North.to_string(), s) => MapDir::North,
            s if sw(MapDir::South.to_string(), s) => MapDir::South,
            s if sw(MapDir::East.to_string(), s) => MapDir::East,
            s if sw(MapDir::West.to_string(), s) => MapDir::West,
            s if sw(MapDir::North.to_string(), s) => MapDir::North,
            s if sw(MapDir::Down.to_string(), s) => MapDir::Down,
            s if sw(MapDir::NorthEast.to_string(), s) => MapDir::NorthEast,
            _ => NoneFound,
        }
    }
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

    pub fn to_string(&self) -> &'static str {
        match self {
            MapDir::North => "north",
            MapDir::South => "south",
            MapDir::East => "east",
            MapDir::West => "west",
            MapDir::Up => "up",
            MapDir::Down => "down",
            MapDir::NorthEast => "northeast",
            MapDir::NoneFound => "",
        }
    }
    pub fn to_string_short(&self) -> &'static str {
        match self {
            MapDir::North => "n",
            MapDir::South => "s",
            MapDir::East => "e",
            MapDir::West => "w",
            MapDir::Up => "u",
            MapDir::Down => "d",
            MapDir::NorthEast => "ne",
            MapDir::NoneFound => "",
        }
    }

    pub fn opposite(&self) -> MapDir {
        match self {
            MapDir::North => MapDir::South,
            MapDir::South => MapDir::North,
            MapDir::East => MapDir::West,
            MapDir::West => MapDir::East,
            MapDir::Up => MapDir::Down,
            MapDir::Down => MapDir::Up,
            MapDir::NorthEast => MapDir::NoneFound,
            MapDir::NoneFound => MapDir::NoneFound,
        }
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
                NoneFound => "",
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
                NoneFound => "",
            }
        )
    }
}
