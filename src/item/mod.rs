pub mod error;
mod handle;

use crate::item::handle::Handle;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use ItemKind::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ItemKind {
    Clothing(Item),
    Weapon(Item),
    Scenery(Item),
    Edible(Item),
    Holdable(Item),
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Item {
    name: String,
    display: String,
    description: String,
    handle: Handle,
}

pub trait Holder {
    fn items(&self) -> &ItemList;
    fn items_mut(&mut self) -> &mut ItemList;

    fn remove_item<S>(&mut self, handle: S) -> Option<ItemKind>
    where
        S: AsRef<str>,
    {
        self.items_mut().get_owned(handle)
    }

    fn give_item(&mut self, item: ItemKind) {
        self.items_mut().push(item);
    }

    fn transfer<H, S>(&mut self, mut other: H, handle: S) -> Result<String, String>
    where
        H: Holder,
        S: AsRef<str>,
    {
        let handle = handle.as_ref();
        let item = match self.remove_item(handle) {
            Some(i) => i,
            None => return Err(handle.to_owned()),
        };

        let name = item.name().to_owned();
        other.give_item(item);
        Ok(name)
    }
}

impl<T> Holder for T
where
    T: AsRef<ItemList> + AsMut<ItemList>,
{
    fn items(&self) -> &ItemList {
        self.as_ref()
    }

    fn items_mut(&mut self) -> &mut ItemList {
        self.as_mut()
    }
}

impl Item {
    pub fn new(name: &str, description: Option<&str>, handle: Handle) -> Self {
        let description = description.unwrap_or_default().to_owned();
        let name = name.to_owned();
        let display = String::new();

        Self {
            name,
            description,
            handle,
            display,
        }
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
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

impl AsRef<ItemList> for ItemList {
    fn as_ref(&self) -> &ItemList {
        self
    }
}

impl AsMut<ItemList> for ItemList {
    fn as_mut(&mut self) -> &mut ItemList {
        self
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

    pub fn get_owned<T: AsRef<str>>(&mut self, handle: T) -> Option<ItemKind> {
        let pos = self.iter().position(|i| i.handle() == handle.as_ref())?;
        Some(self.remove(pos))
    }
}

impl ItemKind {
    pub fn name(&self) -> &str {
        &self.safe_unwrap().name
    }

    pub fn handle(&self) -> &Handle {
        &self.safe_unwrap().handle
    }

    pub fn description(&self) -> &str {
        &self.safe_unwrap().description
    }

    pub fn display(&self) -> &str {
        &self.safe_unwrap().display
    }

    fn safe_unwrap(&self) -> &Item {
        match self {
            Clothing(item) | Weapon(item) | Scenery(item) | Holdable(item) | Edible(item) => &item,
        }
    }
}
