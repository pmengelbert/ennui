use crate::*;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash)]
pub struct Coord(pub i32, pub i32);
impl Coord {
    pub fn north(&self) -> Coord {
        Coord(self.0, self.1 + 1)
    }

    pub fn south(&self) -> Coord {
        Coord(self.0, self.1 - 1)
    }
}

pub struct Map {
    pub map: HashMap<Coord, Room>
}

macro_rules! interpreter {
    ( $( $name:ident ),* ) => {
        {
            let mut i = Interpreter::new();
            $( i.set(stringify!($name), $name); )*
            i
        }
    }
}

pub struct Game {
    pub interpreter: Interpreter,
    pub map: Map,
}

impl Game {
    pub fn new() -> Game {
        let interpreter = interpreter![
            look, 
            say,
            status,
            take,
            inventory,
            drop,
            quit,
            wear,
            remove
        ];

        let mut r1 = Room::new("kitchen".to_string(), "the kitchen".to_string());

        let item = Item {
            kind: ItemType::Normal,
            name: "a book".to_string(),
            description: "a nice book".to_string()
        };
        r1.items.insert("book".to_string(), item);

        let item = Item {
            kind: ItemType::Wearable,
            name: "a shirt".to_string(),
            description: "a nice shirt".to_string()
        };
        r1.items.insert("shirt".to_string(), item);

        let r2 = Room::new("another".to_string(), "the other room".to_string());

        let mut map = HashMap::new();
        map.insert(Coord(0, 0), r1);
        map.insert(Coord(0, 1), r2);

        let map = Map{ map };

        Game { interpreter, map }
    }

    pub fn get_current_room(&mut self, p: &Player) -> &mut Room {
        self.map.map.get_mut(&Coord(0, 0)).unwrap()
    }
}
