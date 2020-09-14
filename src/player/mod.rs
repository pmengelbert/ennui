use self::Status::{Alive, Dead};
use super::item::{Item, ItemType, ItemType::*};

pub struct UUID(String);

pub enum PlayerType<T> {
    Human(T),
    NPC(T),
    Admin(T),
}

pub enum Status {
    Alive,
    Dead,
}

pub struct Player {
    name: String,
    status: Vec<Status>,
    hands: ItemType<Item>,
}

impl Player {
    pub fn new(name: &str) -> Self {
        let (name, status) = (name.to_string(), vec![Alive]);
        let hands: ItemType<Item> = Container(Vec::new());
        Player {
            name,
            status,
            hands,
        }
    }

    pub fn take_item(&mut self, item: ItemType<Item>) -> Result<(), String> {
        if let Container(h) = &mut self.hands {
            h.push(item);
            Ok(())
        } else {
            Err("there's something wrong with your hands... strange".to_string())
        }
    }
}
