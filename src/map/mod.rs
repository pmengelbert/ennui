use std::collections::HashMap;

mod room;
use room::*;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Coord(i32, i32);

pub struct Map {
    m: HashMap<Coord, Room>
}

impl Map {
    pub fn new() -> Self {
        Map {
            m: HashMap::new(),
        }
    }

    pub fn new_test() -> Self {
        let mut m = HashMap::new();
        m.insert(Coord(0, 0), Room::new("the kitchen", "this is the kitchen"));

        Self {
            m,
        }
    }
}
