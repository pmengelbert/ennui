use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use YamlItem::*;

use crate::item::key::Key;
use crate::item::list::{ItemList, ListTrait};
use crate::item::Item::NoItem;
use crate::map::direction::MapDir;
use crate::map::door::{Guard, GuardState};

use crate::attribute::Attribute;
use crate::describe::{Describe, Description};
use crate::gram_object::Hook;

pub mod error;
pub mod handle;
pub mod key;
pub mod list;

/// YamlItem is a no-frills representation of various objects, wrapped in a primary attribute.
/// Its primary use is for serialization
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum YamlItem {
    Clothing(Item2),
    Weapon(Item2),
    Scenery(Item2),
    Edible(Item2),
    Holdable(Item2),
    Guard {
        dir: MapDir,
        state: GuardState,
        info: Item2,
        lock: u64,
    },
    Container(YamlItemList),
    Key(u64, Item2),
}

/// Item is a simple wrapping of an item-y type in a primary attribute
#[derive(Debug)]
pub enum Item {
    Clothing(Item2),
    Weapon(Item2),
    Scenery(Item2),
    Edible(Item2),
    Holdable(Item2),
    Container(Item2, Box<dyn ListTrait<Kind = ItemList>>),
    Guard(MapDir, Box<dyn Guard<Lock = u64, Kind = ItemList>>),
    Key(Box<dyn Key<u64>>),
    NoItem,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Item2 {
    #[serde(default)]
    pub info: Description,
    #[serde(default)]
    pub attr: Vec<Quality>,
}

impl Describe for Item2 {
    fn handle(&self) -> Hook {
        self.info.handle()
    }

    fn name(&self) -> String {
        self.info.name()
    }

    fn description(&self) -> String {
        self.info.description()
    }

    fn display(&self) -> String {
        self.info.display()
    }
}

impl Default for Item {
    fn default() -> Self {
        NoItem
    }
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

impl Attribute<Quality> for Vec<Quality> {
    fn attr(&self) -> Vec<Quality> {
        self.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.push(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        let count = self.iter().take_while(|i| **i != q).count();

        if count < self.len() {
            self.remove(count);
        }
    }
}

impl Describe for Item {
    fn name(&self) -> String {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.info.name(),
            Container(_, i) => i.name(),
            Key(i) => i.name(),
            Guard(_, i) => i.name(),
            NoItem => String::new(),
        }
    }

    fn display(&self) -> String {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.info.display(),
            Container(_, i) => i.display(),
            Key(i) => i.display(),
            Guard(_, i) => i.display(),
            NoItem => String::new(),
        }
    }

    fn description(&self) -> String {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.info.description(),
            Container(_, i) => i.description(),
            Key(i) => i.description(),
            Guard(_, i) => i.description(),
            NoItem => String::new(),
        }
    }

    fn handle(&self) -> Hook {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.info.handle(),
            Container(_, i) => i.handle(),
            Key(i) => i.handle(),
            Guard(_, i) => i.handle(),
            NoItem => Hook::default(),
        }
    }
}

impl Attribute<Quality> for Item2 {
    fn attr(&self) -> Vec<Quality> {
        self.attr.clone()
    }

    fn unset_attr(&mut self, q: Quality) {
        self.attr.unset_attr(q)
    }

    fn set_attr(&mut self, q: Quality) {
        self.attr.set_attr(q)
    }
}

impl Attribute<Quality> for Item {
    fn attr(&self) -> Vec<Quality> {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.attr(),
            Container(i, _) => i.attr(),
            Key(i) => i.attr(),
            Guard(_, i) => i.attr(),
            NoItem => vec![],
        }
    }

    fn set_attr(&mut self, q: Quality) {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.set_attr(q),
            Container(i, _) => i.set_attr(q),
            Key(i) => i.set_attr(q),
            Guard(_, i) => i.set_attr(q),
            NoItem => (),
        }
    }

    fn unset_attr(&mut self, q: Quality) {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.unset_attr(q),
            Container(i, _) => i.unset_attr(q),
            Key(i) => i.unset_attr(q),
            Guard(_, i) => i.unset_attr(q),
            NoItem => (),
        }
    }
}

impl Default for YamlItem {
    fn default() -> Self {
        Holdable(Item2::default())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct YamlItemList {
    inner: Vec<YamlItem>,
    #[serde(default, flatten)]
    info: Item2,
}

impl Attribute<Quality> for YamlItemList {
    fn attr(&self) -> Vec<Quality> {
        self.info.attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.info.attr.set_attr(q)
    }

    fn unset_attr(&mut self, q: Quality) {
        self.info.attr.unset_attr(q);
    }
}

impl Describe for YamlItemList {
    fn name(&self) -> String {
        self.info.info.name()
    }

    fn display(&self) -> String {
        self.info.info.display()
    }

    fn description(&self) -> String {
        self.info.info.description()
    }

    fn handle(&self) -> Hook {
        self.info.info.handle()
    }
}

impl YamlItemList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            info: Item2 {
                info: Description {
                    name: "".to_string(),
                    display: "".to_string(),
                    description: "".to_string(),
                    handle: Default::default(),
                },
                attr: vec![Quality::Container],
            },
        }
    }
    pub fn get(&self, handle: &str) -> Option<&YamlItem> {
        self.inner.iter().find(|i| i.handle() == handle)
    }

    pub fn get_owned<T: AsRef<str>>(&mut self, handle: T) -> Option<YamlItem> {
        let pos = self
            .inner
            .iter()
            .position(|i| i.handle() == handle.as_ref())?;
        Some(self.inner.remove(pos))
    }
}

impl YamlItem {
    fn safe_unwrap(&self) -> &Item2 {
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

    fn safe_unwrap_mut(&mut self) -> &mut Item2 {
        match self {
            Key(_, item)
            | Clothing(item)
            | Weapon(item)
            | Scenery(item)
            | Holdable(item)
            | Edible(item) => item,
            Container(i) => &mut i.info,
            YamlItem::Guard { info, .. } => info,
        }
    }
}

impl Describe for YamlItem {
    fn name(&self) -> String {
        self.safe_unwrap().name()
    }

    fn display(&self) -> String {
        self.safe_unwrap().display()
    }

    fn description(&self) -> String {
        self.safe_unwrap().description()
    }

    fn handle(&self) -> Hook {
        self.safe_unwrap().handle()
    }
}

impl Attribute<Quality> for YamlItem {
    fn attr(&self) -> Vec<Quality> {
        self.safe_unwrap().attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.safe_unwrap_mut().set_attr(q)
    }

    fn unset_attr(&mut self, q: Quality) {
        self.safe_unwrap_mut().unset_attr(q)
    }
}
