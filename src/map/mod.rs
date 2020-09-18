use std::collections::HashMap;

pub mod room;
use room::*;
use super::item::{ItemType, Item, ItemType::*};

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct Coord(pub i32, pub i32);

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
        let mut room = Room::new("the kitchen", "this is the kitchen");
        let mut room2 = Room::new("the hallway", "this is the hallway");
        let item = Item::new("knife", "a knife");

        room.add_item(Weapon(item));
        m.insert(Coord(0, 0), room);
        m.insert(Coord(0, 1), room2);

        Self {
            m,
        }
    }

    pub fn get(&self, c: Coord) -> Option<&Room> {
        self.m.get(&c)
    }

    pub fn get_mut(&mut self, c: Coord) -> Option<&mut Room> {
        self.m.get_mut(&c)
    }
}
