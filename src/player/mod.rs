use self::Status::{Alive, Dead};
use super::item::{Item, ItemType, ItemType::*};
use uuid;

#[test]
fn test_uuid() {
    let my_uuid = uuid::Uuid::new_v4();
    let my_uuid2 = uuid::Uuid::new_v4();

    assert_ne!(my_uuid, my_uuid2);
}

#[derive(Copy, Eq, PartialEq, Clone, Hash)]
pub struct UUID(u128);

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
    uuid: UUID,
}

impl Player {
    pub fn new(name: &str) -> Self {
        let (name, status) = (name.to_string(), vec![Alive]);
        let hands: ItemType<Item> = Container(Vec::new());
        let uuid = UUID(uuid::Uuid::new_v4().as_u128());
        Player {
            name,
            status,
            hands,
            uuid,
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

    pub fn uuid(&self) -> UUID {
        self.uuid
    }
}
