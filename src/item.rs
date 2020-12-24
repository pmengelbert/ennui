use crate::PassFail;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use ItemKind::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum ItemKind {
    Clothing(Item),
    Weapon(Item),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Item {
    name: String,
    description: String,
    handle: String,
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
    fn transfer<H, S>(&mut self, mut other: H, handle: S) -> PassFail
    where
        H: Holder,
        S: AsRef<str>,
    {
        let item = self.remove_item(handle)?;
        other.give_item(item);
        Ok(())
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
    pub fn new(name: &str, description: Option<&str>, handle: &str) -> Self {
        let description = description.unwrap_or("").to_owned();
        let name = name.to_owned();
        let handle = handle.to_owned();

        Self {
            name,
            handle,
            description,
        }
    }

    pub fn handle(&self) -> &str {
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
