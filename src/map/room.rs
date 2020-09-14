use super::super::item::{ItemType::*, ItemType, Item};
use std::collections::HashMap;

pub struct Room {
    name: String,
    description: String,
    items: ItemType<Item>,
}

impl Room {
    pub fn new(name: &str, description: &str) -> Self {
        let (name, description) = (name.to_string(), description.to_string());
        let items = Container(Vec::new());
        Self {
            name,
            description,
            items,
        }
    }
}
