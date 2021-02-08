pub mod direction;
use direction::MapDir;

use serde::{Deserialize, Serialize};

pub trait Locate {
    fn loc(&self) -> Coord;
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Hash, Default, Clone, Copy)]
pub struct Coord(pub i64, pub i64);

impl Coord {
    pub fn north(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x, *y + 1)
    }

    pub fn south(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x, *y - 1)
    }

    pub fn east(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x + 1, *y)
    }

    pub fn west(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x - 1, *y)
    }

    pub fn add(&self, dir: MapDir) -> Option<Self> {
        use MapDir::*;

        Some(match dir {
            North => self.north(),
            South => self.south(),
            East => self.east(),
            West => self.west(),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod coord_test {
    use super::*;

    #[test]
    fn coord_test_north() {
        assert_eq!(Coord(0, 0).north(), Coord(0, 1));
    }

    #[test]
    fn coord_test_south() {
        assert_eq!(Coord(0, 0).south(), Coord(0, -1));
    }

    #[test]
    fn coord_test_east() {
        assert_eq!(Coord(0, 0).east(), Coord(1, 0));
    }

    #[test]
    fn coord_test_west() {
        assert_eq!(Coord(0, 0).west(), Coord(-1, 0));
    }
}
