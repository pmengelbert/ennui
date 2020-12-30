use std::borrow::{Borrow, BorrowMut};
use std::mem::take;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use serde::export::fmt::Debug;

use YamlItem::*;

use crate::item::handle::Handle;
use crate::item::key::{Key, KeyType};
use crate::map::direction::MapDir;
use crate::map::door::{Guard, GuardState, RenaissanceGuard};
use crate::item::list::{ItemListTrait, ItemList};

pub mod error;
pub mod handle;
pub mod key;
pub mod list;

pub trait Describe: Send + Sync + Debug + Attribute<Quality> {
    fn name(&self) -> &str;
    fn display(&self) -> &str;
    fn description(&self) -> &str;
    fn handle(&self) -> &Handle;
}

/// YamlItem is a no-frills representation of various objects, wrapped in a primary attribute.
/// Its primary use is for serialization
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum YamlItem {
    Clothing(Description),
    Weapon(Description),
    Scenery(Description),
    Edible(Description),
    Holdable(Description),
    Guard {
        dir: MapDir,
        state: GuardState,
        info: Description,
        lock: u64,
    },
    Container(YamlItemList),
    Key(u64, Description),
}

/// Item is a simple wrapping of an item-y type in a primary attribute
#[derive(Debug)]
pub enum Item {
    Clothing(Box<dyn Describe>),
    Weapon(Box<dyn Describe>),
    Scenery(Box<dyn Describe>),
    Edible(Box<dyn Describe>),
    Holdable(Box<dyn Describe>),
    Container(Box<dyn ItemListTrait<Kind = ItemList>>),
    Guard(MapDir, Box<dyn Guard<Lock = u64, Kind = ItemList>>),
    Key(Box<dyn Key<u64>>),
}

/// `Quality` is used to describe extra attributes on players, items and rooms
#[derive(Copy, Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub enum Quality {
    Clothing,
    Weapon,
    Scenery,
    Edible,
    Holdable,
    Container,
    Guard,
    Key,
}

pub trait Attribute<T: Copy + Eq> {
    fn attr(&self) -> &[T];

    fn is(&self, a: T) -> bool {
        self.attr().contains(&a)
    }

    fn is_all(&self, ats: &[T]) -> bool {
        for a in ats {
            if !self.attr().contains(a) {
                return false;
            }
        }

        true
    }
}

impl Describe for Item {
    fn name(&self) -> &str {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.name(),
            Container(i) => i.name(),
            Key(i) => i.name(),
            Guard(_, i) => i.name(),
        }
    }

    fn display(&self) -> &str {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.display(),
            Container(i) => i.display(),
            Key(i) => i.display(),
            Guard(_, i) => i.display(),
        }
    }

    fn description(&self) -> &str {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.description(),
            Container(i) => i.description(),
            Key(i) => i.description(),
            Guard(_, i) => i.description(),
        }
    }

    fn handle(&self) -> &Handle {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.handle(),
            Container(i) => i.handle(),
            Key(i) => i.handle(),
            Guard(_, i) => i.handle(),
        }
    }
}

impl Attribute<Quality> for Item {
    fn attr(&self) -> &[Quality] {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.attr(),
            Container(i) => i.attr(),
            Key(i) => i.attr(),
            Guard(_, i) => i.attr(),
        }
    }
}

impl Default for YamlItem {
    fn default() -> Self {
        Holdable(Description::default())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Description {
    pub name: String,
    pub display: String,
    pub description: String,
    pub handle: Handle,
    #[serde(default)]
    pub attributes: Vec<Quality>,
}

impl Describe for Description {
    fn name(&self) -> &str {
        &self.name
    }

    fn display(&self) -> &str {
        &self.display
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl Attribute<Quality> for Description {
    fn attr(&self) -> &[Quality] {
        &self.attributes
    }
}

impl Description {
    pub fn new(name: &str, description: Option<&str>, handle: Handle) -> Self {
        let description = description.unwrap_or_default().to_owned();
        let name = name.to_owned();
        let display = String::new();
        let attributes = Vec::new();

        Self {
            name,
            description,
            handle,
            display,
            attributes,
        }
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct YamlItemList {
    inner: Vec<YamlItem>,
    info: Description,
}

impl Attribute<Quality> for YamlItemList {
    fn attr(&self) -> &[Quality] {
        &self.info.attributes
    }
}

impl Deref for YamlItemList {
    type Target = Vec<YamlItem>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for YamlItemList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl AsRef<YamlItemList> for YamlItemList {
    fn as_ref(&self) -> &YamlItemList {
        self
    }
}

impl AsMut<YamlItemList> for YamlItemList {
    fn as_mut(&mut self) -> &mut YamlItemList {
        self
    }
}

impl Describe for YamlItemList {
    fn name(&self) -> &str {
        &self.info.name()
    }

    fn display(&self) -> &str {
        &self.info.display()
    }

    fn description(&self) -> &str {
        &self.info.description()
    }

    fn handle(&self) -> &Handle {
        &self.info.handle()
    }
}

impl YamlItemList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            info: Description {
                name: "".to_string(),
                display: "".to_string(),
                description: "".to_string(),
                handle: Default::default(),
                attributes: vec![Quality::Container],
            },
        }
    }
    pub fn get(&self, handle: &str) -> Option<&YamlItem> {
        self.iter().find(|i| i.handle() == handle)
    }

    pub fn get_owned<T: AsRef<str>>(&mut self, handle: T) -> Option<YamlItem> {
        let pos = self.iter().position(|i| i.handle() == handle.as_ref())?;
        Some(self.remove(pos))
    }
}

impl YamlItem {
    fn safe_unwrap(&self) -> &Description {
        match self {
            Key(_, item)
            | Clothing(item)
            | Weapon(item)
            | Scenery(item)
            | Holdable(item)
            | Edible(item) => &item,
            Container(i) => &i.info,
            YamlItem::Guard { info, .. } => &info,
        }
    }
}

impl Describe for YamlItem {
    fn name(&self) -> &str {
        &self.safe_unwrap().name()
    }

    fn display(&self) -> &str {
        &self.safe_unwrap().display()
    }

    fn description(&self) -> &str {
        &self.safe_unwrap().description()
    }

    fn handle(&self) -> &Handle {
        &self.safe_unwrap().handle()
    }
}

impl Attribute<Quality> for YamlItem {
    fn attr(&self) -> &[Quality] {
        self.safe_unwrap().attr()
    }
}

#[cfg(test)]
mod item_trait_test {
    

    #[test]
    fn item_trait_test_1() {
        // let x: Vec<Box<ItemListTrait>> = Vec::new();
    }
}
