use std::collections::HashMap;

pub mod room;
mod io;
use super::item::{Item, ItemType, ItemType::*};
use room::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct Coord(pub i32, pub i32);

pub struct Map {
    m: HashMap<Coord, Room>,
}

impl Map {
    pub fn new() -> Self {
        Map { m: HashMap::new() }
    }

    pub fn from_file(filename: &str) -> Self {
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(err) => panic!(err),
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(_) => (),
        }

        let map: HashMap<usize, LightRoom> = match serde_yaml::from_str(&contents) {
            Ok(o) => o,
            Err(err) => panic!(err),
        };

        let mut ret = HashMap::new();
        for (_, v) in map {
            let c = Coord(v.x, v.y);
            let mut r = Room::new(&v.name, &v.description);
            for item in v.items {
                r.add_item(item);
            }

            ret.insert(c, r);
        }

        Map { m: ret }
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

#[test]
fn test_yaml() {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = match File::open("./map/01.yaml") {
        Ok(f) => f,
        Err(err) => panic!(err),
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(_) => (),
    }

    let map: HashMap<usize, LightRoom> = match serde_yaml::from_str(&contents) {
        Ok(o) => o,
        Err(err) => panic!(err),
    };

    let x = &map[&1];
    println!("{}\n---------\n{}\n{:?}", x.name, x.description, x.items);
}

#[derive(Deserialize)]
struct LightRoom {
    name: String,
    x: i32,
    y: i32,
    description: String,
    items: Vec<ItemType<Item>>,
}
