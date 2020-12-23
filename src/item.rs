use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum ItemKind {
    Clothing(Item),
    Weapon(Item),
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct ItemList(Vec<ItemKind>);

impl Deref for ItemList {
    type Target = Vec<ItemKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ItemList {
    pub fn new() -> Self {
        ItemList(Vec::new())
    }
    pub fn get(&self, handle: &str) -> Option<&ItemKind> {
        self.iter().find(|i| i.handle() == handle)
    }

    pub fn get_mut(&mut self, handle: &str) -> Option<&mut ItemKind> {
        self.iter_mut().find(|i| i.handle() == handle)
    }

    pub fn get_owned(&mut self, handle: &str) -> Option<ItemKind> {
        let pos = self.iter().position(|i| i.handle() == handle)?;
        Some(self.remove(pos))
    }
}

use ItemKind::*;
impl ItemKind {
    pub fn name(&self) -> &str {
        &self.safe_unwrap().name
    }

    pub fn handle(&self) -> &str {
        &self.safe_unwrap().handle
    }

    pub fn description(&self) -> &str {
        &self.safe_unwrap().description
    }

    fn safe_unwrap(&self) -> &Item {
        match self {
            Clothing(item) | Weapon(item) => &item,
        }
    }
}
