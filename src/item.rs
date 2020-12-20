use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum ItemKind {
    Clothing(Item),
    Weapon(Item),
}

#[derive(Debug)]
pub struct Item {
    name: String,
    description: String,
    handle: String,
}

impl Item {
    pub fn new(name: &str, description: Option<&str>, handle: &str) -> Self {
        let description = description.unwrap_or("").to_owned();
        let name = name.to_owned();
        let handle = handle.to_owned();
        Item {
            name,
            handle,
            description,
        }
    }

    pub fn handle(&self) -> &str {
        &self.handle
    }
}

#[derive(Debug)]
pub struct ItemList {
    list: Vec<ItemKind>,
}

impl Deref for ItemList {
    type Target = Vec<ItemKind>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for ItemList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl ItemList {
    pub fn new() -> Self {
        ItemList {
            list: Vec::new(),
        }
    }
    pub fn push(&mut self, item: ItemKind) {
        self.list.push(item)
    }

    pub fn get (&self, handle: &str) -> Option<&ItemKind> {
        self.list.iter().find(|i| i.handle() == handle)
    }

    pub fn get_mut (&mut self, handle: &str) -> Option<&mut ItemKind> {
        self.list.iter_mut().find(|i| i.handle() == handle)
    }

    pub fn get_owned(&mut self, handle: &str) -> Option<ItemKind> {
        let pos = self.list.iter().position(|i| i.handle() == handle)?;
        Some(self.list.remove(pos))
    }
}

use ItemKind::*;
impl ItemKind {
    pub fn name(&self) -> &str {
        &self.unwrap().name
    }

    pub fn handle(&self) -> &str {
        &self.unwrap().handle
    }

    pub fn description(&self) -> &str {
        &self.unwrap().description
    }

    fn unwrap(&self) -> &Item {
        match self {
            Clothing(item) => &item,
            Weapon(item) => &item,
        }
    }
}