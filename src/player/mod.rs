use self::Status::{Alive, Dead};
use self::{PlayerType::*};
use super::item::{Item, ItemType, ItemType::*};
use super::map::{Coord};
use uuid;

#[test]
fn test_uuid() {
    let my_uuid = uuid::Uuid::new_v4();
    let my_uuid2 = uuid::Uuid::new_v4();

    assert_ne!(my_uuid, my_uuid2);
}

#[derive(Copy, Eq, PartialEq, Clone, Hash, Debug)]
pub struct UUID(u128);

impl UUID {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub enum PlayerType<T> {
    Human(T),
    NPC(T),
    Admin(T),
}

impl PlayerType<Player> {
    pub fn player(&self) -> &Player {
        match &self {
            Human(ref p) | NPC(ref p) | Admin(ref p) => p,
        }
    }

    pub fn player_mut(&mut self) -> &mut Player {
        match self {
            &mut Human(ref mut p) | &mut NPC(ref mut p) | &mut Admin(ref mut p) => p,
        }
    }
}

#[derive(Debug)]
pub enum Status {
    Alive,
    Dead,
}

#[derive(Debug)]
pub struct Player {
    name: String,
    status: Vec<Status>,
    hands: ItemType<Item>,
    uuid: UUID,
    location: Coord,
    description: String,
}

impl Player {
    pub fn new(name: &str) -> Self {
        let (name, status) = (name.to_string(), vec![Alive]);
        let hands: ItemType<Item> = Container(Vec::new());
        let uuid = UUID(uuid::Uuid::new_v4().as_u128());
        let location = Coord(0, 0);
        let description = String::new();
        Player {
            name,
            status,
            hands,
            uuid,
            location,
            description,
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

    pub fn location(&self) -> Coord {
        self.location
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn hands(&self) -> &ItemType<Item> {
        &self.hands
    }

    pub fn hands_mut(&mut self) -> &mut ItemType<Item> {
        &mut self.hands
    }

    pub fn set_location(&mut self, c: Coord) {
        self.location = c;
    }
}
