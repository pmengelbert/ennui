use super::super::item::{Item, ItemType, ItemType::*};
use super::super::player::{Player, PlayerType, PlayerType::*, UUID};

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub description: String,
    pub items: ItemType<Item>,
    pub players: Vec<UUID>,
}

impl Room {
    pub fn new(name: &str, description: &str) -> Self {
        let (name, description) = (name.to_string(), description.to_string());
        let items = Container(Vec::new());
        let players = Vec::new();
        Self {
            name,
            description,
            items,
            players,
        }
    }

    pub fn to_string(&self) -> (String, String) {
        let mut name_and_description = String::new();
        let mut items = String::new();

        name_and_description.push_str(&self.name);
        name_and_description.push_str("\n-----------------------\n");
        name_and_description.push_str(&self.description);

        items.push_str(&self.items.to_string());

        (name_and_description, items)
    }

    pub fn add_item(&mut self, i: ItemType<Item>) -> Result<(), String> {
        match self.items.container_mut() {
            Some(ref mut c) => {
                c.push(i);
                Ok(())
            }
            None => Err("Item not found".to_string()),
        }
    }

    pub fn add_player(&mut self, uuid: UUID) {
        self.players.push(uuid);
    }

    pub fn remove_player(&mut self, uuid: UUID) -> Result<(), String> {
        self.players = self
            .players
            .split(|&u| u == uuid)
            .flatten()
            .map(|x| *x)
            .collect();

        Ok(())
    }

    pub fn items(&mut self) -> &mut ItemType<Item> {
        &mut self.items
    }

    pub fn items_not_mut(&self) -> &ItemType<Item> {
        &self.items
    }
}
