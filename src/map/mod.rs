use std::collections::HashMap;

pub mod room;
use super::item::{Item, ItemType, ItemType::*};
use room::*;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct Coord(pub i32, pub i32);

pub struct Map {
    m: HashMap<Coord, Room>,
}

impl Map {
    pub fn new() -> Self {
        Map { m: HashMap::new() }
    }

    pub fn new_test() -> Self {
        let mut m = HashMap::new();
        let mut room = Room::new("the kitchen", "this is the kitchen");
        let mut room2 = Room::new("the hallway", "this is the hallway");
        let item = Item::new(
            "a knife",
            "knife",
            "a ceremonial knife, for when you have to poop and stuff",
        );

        let item2 = Item::new(
            "a pink shirt",
            "shirt",
            "it's an ugly pink shirt. you'd rather not wear it...",
        );

        room.add_item(Weapon(item));
        room.add_item(Armor(item2));
        m.insert(Coord(0, 0), room);
        m.insert(Coord(0, 1), room2);

        Self { m }
    }

    pub fn get(&self, c: Coord) -> Option<&Room> {
        self.m.get(&c)
    }

    pub fn get_mut(&mut self, c: Coord) -> Option<&mut Room> {
        self.m.get_mut(&c)
    }
}
