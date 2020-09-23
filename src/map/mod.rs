use std::collections::HashMap;

mod io;
pub mod room;
use super::item::{Item, ItemType};
use room::*;
use serde::Deserialize;
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

    pub fn from_file(filename: &str) -> Result<Self, String> {
        let mut file = File::open(filename).expect(&format!("cannot find file {}", filename));

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
                r.add_item(item)?;
            }

            ret.insert(c, r);
        }

        Ok(Map { m: ret })
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
