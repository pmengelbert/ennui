pub mod error;
mod handle;
pub mod key;

use crate::item::handle::Handle;
use serde::export::fmt::Debug;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use ItemKind::*;
use std::borrow::Borrow;
use crate::item::key::Key;

pub trait ItemTrait : Send + Sync {
    fn name(&self) -> &str;
    fn display(&self) -> &str;
    fn description(&self) -> &str;
    fn kind(&self) -> ItemKind;
    fn handle(&self) -> &Handle;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ItemKind {
    Clothing(Item),
    Weapon(Item),
    Scenery(Item),
    Edible(Item),
    Holdable(Item),
    Container,
}

pub enum ItemKind2 {
    Clothing(Box<dyn ItemTrait>),
    Weapon(Box<dyn ItemTrait>),
    Scenery(Box<dyn ItemTrait>),
    Edible(Box<dyn ItemTrait>),
    Holdable(Box<dyn ItemTrait>),
    Container(Box<dyn ItemTrait>),
    Key(Box<dyn Key<u64>>)
}

impl ItemTrait for ItemKind2 {
    fn name(&self) -> &str {
        use ItemKind2::*;
        match self {
            Clothing(i)
            | Weapon(i)
            | Scenery(i)
            | Edible(i)
            | Holdable(i)
            | Container(i) => i.name(),
            Key(i) => i.name(),
        }
    }

    fn display(&self) -> &str {
        use ItemKind2::*;
        match self {
            Clothing(i)
            | Weapon(i)
            | Scenery(i)
            | Edible(i)
            | Holdable(i)
            | Container(i) => i.display(),
            Key(i) => i.display(),
        }
    }

    fn description(&self) -> &str {
        use ItemKind2::*;
        match self {
            Clothing(i)
            | Weapon(i)
            | Scenery(i)
            | Edible(i)
            | Holdable(i)
            | Container(i) => i.description(),
            Key(i) => i.description(),
        }
    }

    fn kind(&self) -> ItemKind {
        use ItemKind::*;
        Clothing(Item::default())
    }

    fn handle(&self) -> &Handle {
        use ItemKind2::*;
        match self {
            Clothing(i)
            | Weapon(i)
            | Scenery(i)
            | Edible(i)
            | Holdable(i)
            | Container(i) => i.handle(),
            Key(i) => i.handle(),
        }
    }
}

impl Default for ItemKind {
    fn default() -> Self {
        Holdable(Item::default())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Item {
    name: String,
    display: String,
    description: String,
    handle: Handle,
}

pub trait Holder {
    fn items(&self) -> &ItemList2;
    fn items_mut(&mut self) -> &mut ItemList2;

    fn transfer<H, S>(&mut self, mut other: H, handle: S) -> Result<String, String>
    where
        H: Holder,
        S: AsRef<str>,
    {
        let handle = handle.as_ref();
        let item = match self.items_mut().get_owned(handle) {
            Some(i) => i,
            None => return Err(handle.to_owned()),
        };

        let name = item.name().to_owned();
        other.items_mut().insert(item);
        Ok(name)
    }
}

impl<T> Holder for T
where
    T: AsRef<ItemList2> + AsMut<ItemList2>,
{
    fn items(&self) -> &ItemList2 {
        self.as_ref()
    }

    fn items_mut(&mut self) -> &mut ItemList2 {
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

    pub fn get_owned<T: AsRef<str>>(&mut self, handle: T) -> Option<ItemKind> {
        let pos = self.iter().position(|i| i.handle() == handle.as_ref())?;
        Some(self.remove(pos))
    }
}

impl ItemKind {
    fn safe_unwrap(&self) -> Option<&Item> {
        match self {
            Clothing(item) | Weapon(item) | Scenery(item) | Holdable(item) | Edible(item) => Some(&item),
            Container => None
        }
    }
}

impl ItemTrait for ItemKind {
    fn name(&self) -> &str {
        self.safe_unwrap().map(|i| i.name.as_str()).unwrap_or_default()
    }

    fn display(&self) -> &str {
        &self.safe_unwrap().map(|i| i.display.as_str()).unwrap_or_default()
    }

    fn description(&self) -> &str {
        &self.safe_unwrap().map(|i| i.description.as_str()).unwrap_or_default()
    }

    fn kind(&self) -> ItemKind {
        self.clone()
    }

    fn handle(&self) -> &Handle {
        &self.safe_unwrap().map(|i| i.handle()).as_ref().unwrap()
    }
}

pub trait ItemListTrait : ItemTrait {
    fn get(&self, handle: &str) -> Option<&dyn ItemTrait>;
    fn get_owned(&mut self, handle: &str) -> Option<Box<ItemTrait>>;
    fn insert(&mut self, item: Box<dyn ItemTrait>);

    fn transfer(&mut self, other: &mut Self, handle: &str) -> Result<String, String> {
        let handle = handle.as_ref();
        let item = match self.get_owned(handle) {
            Some(i) => i,
            None => return Err(handle.to_owned()),
        };

        let name = item.name().to_owned();
        other.insert(item);
        Ok(name)
    }
}

#[derive(Default)]
pub struct ItemList2 {
    inner: Vec<Box<dyn ItemTrait>>,
    info: ItemKind,
}

impl Deref for ItemList2 {
    type Target = Vec<Box<dyn ItemTrait>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ItemList2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ItemTrait for ItemList2 {
    fn name(&self) -> &str {
        self.info.name()
    }

    fn display(&self) -> &str {
        self.info.display()
    }

    fn description(&self) -> &str {
        self.info.description()
    }

    fn kind(&self) -> ItemKind {
        Container
    }

    fn handle(&self) -> &Handle {
        self.info.handle()
    }
}


impl ItemListTrait for ItemList2 {
    fn get(&self, handle: &str) -> Option<&dyn ItemTrait> {
        self.iter().find(|i| i.handle() == handle).map(|i| i.borrow())
    }

    fn get_owned(&mut self, handle: &str) -> Option<Box<dyn ItemTrait>> {
        let pos = self.iter().position(|i| i.handle() == handle)?;
        Some(self.remove(pos))
    }

    fn insert(&mut self, item: Box<dyn ItemTrait>) {
        self.inner.push(item);
    }
}

impl ItemList2 {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            info: Default::default(),
        }
    }
}

impl From<ItemList> for ItemList2 {
    fn from(l: ItemList) -> Self {
        let mut v: Vec<Box<dyn ItemTrait>> = Vec::new();
        for i in &*l {
            v.push(Box::new(i.clone()));
        }
        ItemList2 {
            inner: v,
            info: Default::default()
        }
    }
}

#[cfg(test)]
mod item_trait_test {
   use super::*;

    #[test]
    fn item_trait_test_1() {
        let x: Vec<Box<ItemListTrait>> = Vec::new();
    }
}