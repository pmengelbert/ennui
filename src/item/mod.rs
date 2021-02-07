use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use YamlItem::*;

use crate::item::handle::Hook;
use crate::item::key::Key;
use crate::item::list::{ItemList, ListTrait};
use crate::item::Item::NoItem;
use crate::map::direction::MapDir;
use crate::map::door::{Guard, GuardState};

pub mod error;
pub mod handle;
pub mod key;
pub mod list;

pub trait Describe: Send + Sync + Debug + Attribute<Quality> {
    fn name(&self) -> String;
    fn display(&self) -> String;
    fn description(&self) -> String;
    fn handle(&self) -> Hook;
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
    Container(Box<dyn ListTrait<Kind = ItemList>>),
    Guard(MapDir, Box<dyn Guard<Lock = u64, Kind = ItemList>>),
    Key(Box<dyn Key<u64>>),
    NoItem,
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

pub trait Attribute<T: Copy + Eq> {
    fn attr(&self) -> Vec<T>;
    fn set_attr(&mut self, q: T);
    fn unset_attr(&mut self, q: T);

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
    fn name(&self) -> String {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.name(),
            Container(i) => i.name(),
            Key(i) => i.name(),
            Guard(_, i) => i.name(),
            NoItem => String::new(),
        }
    }

    fn display(&self) -> String {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.display(),
            Container(i) => i.display(),
            Key(i) => i.display(),
            Guard(_, i) => i.display(),
            NoItem => String::new(),
        }
    }

    fn description(&self) -> String {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.description(),
            Container(i) => i.description(),
            Key(i) => i.description(),
            Guard(_, i) => i.description(),
            NoItem => String::new(),
        }
    }

    fn handle(&self) -> Hook {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.handle(),
            Container(i) => i.handle(),
            Key(i) => i.handle(),
            Guard(_, i) => i.handle(),
            NoItem => Hook::default(),
        }
    }
}

impl Attribute<Quality> for Item {
    fn attr(&self) -> Vec<Quality> {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.attr(),
            Container(i) => i.attr(),
            Key(i) => i.attr(),
            Guard(_, i) => i.attr(),
            NoItem => vec![],
        }
    }

    fn set_attr(&mut self, q: Quality) {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.set_attr(q),
            Container(i) => i.set_attr(q),
            Key(i) => i.set_attr(q),
            Guard(_, i) => i.set_attr(q),
            NoItem => (),
        }
    }

    fn unset_attr(&mut self, q: Quality) {
        use Item::*;
        match self {
            Clothing(i) | Weapon(i) | Scenery(i) | Edible(i) | Holdable(i) => i.unset_attr(q),
            Container(i) => i.unset_attr(q),
            Key(i) => i.unset_attr(q),
            Guard(_, i) => i.unset_attr(q),
            NoItem => (),
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
    pub handle: Hook,
    #[serde(default)]
    pub attributes: Vec<Quality>,
}

impl Describe for Description {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn display(&self) -> String {
        self.display.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn handle(&self) -> Hook {
        self.handle.clone()
    }
}

impl Attribute<Quality> for Description {
    fn attr(&self) -> Vec<Quality> {
        self.attributes.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.attributes.push(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        let pos = self.attributes.iter().position(|u| *u == q);
        if let Some(pos) = pos {
            self.attributes.remove(pos);
        }
    }
}

impl Description {
    pub fn new(name: &str, description: Option<&str>, handle: Hook) -> Self {
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
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct YamlItemList {
    inner: Vec<YamlItem>,
    #[serde(default)]
    info: Description,
}

impl Attribute<Quality> for YamlItemList {
    fn attr(&self) -> Vec<Quality> {
        self.info.attributes.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.info.set_attr(q)
    }

    fn unset_attr(&mut self, q: Quality) {
        self.info.unset_attr(q);
    }
}

impl Describe for YamlItemList {
    fn name(&self) -> String {
        self.info.name()
    }

    fn display(&self) -> String {
        self.info.display()
    }

    fn description(&self) -> String {
        self.info.description()
    }

    fn handle(&self) -> Hook {
        self.info.handle()
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

    fn safe_unwrap_mut(&mut self) -> &mut Description {
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
        self.safe_unwrap().attr()
    }

    fn set_attr(&mut self, q: Quality) {
        self.safe_unwrap_mut().set_attr(q)
    }

    fn unset_attr(&mut self, q: Quality) {
        self.safe_unwrap_mut().unset_attr(q)
    }
}
