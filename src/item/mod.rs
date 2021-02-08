use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::attribute::{Attribute, Quality};
use crate::describe::{Describe, Description};
use crate::hook::Hook;
use crate::location::direction::MapDir;
use crate::obstacle::door::GuardState;
use crate::obstacle::key::{Key, KeyType};
use list::{Guard, ItemList, ListTrait};
use Item::NoItem;

pub mod list;

pub trait ItemDescribe: Describe + Attribute<Quality> {}
pub trait ListDescribe: Describe + Attribute<Quality> + ListTrait {}
pub trait GuardDescribe: Describe + Attribute<Quality> + Guard {}

impl ListDescribe for ItemList {}

/// YamlItem is a no-frills representation of various objects, wrapped in a primary attribute.
/// Its primary use is for serialization
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum YamlItem {
    Clothing(DescriptionWithQualities),
    Weapon(DescriptionWithQualities),
    Scenery(DescriptionWithQualities),
    Edible(DescriptionWithQualities),
    Holdable(DescriptionWithQualities),
    Guard {
        dir: MapDir,
        state: GuardState,
        info: DescriptionWithQualities,
        lock: u64,
    },
    Container(YamlItemList),
    Key(u64, DescriptionWithQualities),
}

/// Item is a simple wrapping of an item-y type in a primary attribute
#[derive(Debug)]
pub enum Item {
    Clothing(Box<dyn ItemDescribe>),
    Weapon(Box<dyn ItemDescribe>),
    Scenery(Box<dyn ItemDescribe>),
    Edible(Box<dyn ItemDescribe>),
    Holdable(Box<dyn ItemDescribe>),
    Container(Box<dyn ListDescribe<Kind = ItemList>>),
    Guard(MapDir, Box<dyn GuardDescribe<Lock = u64, Kind = ItemList>>),
    Key(Box<dyn Key<u64>>),
    NoItem,
}

impl Default for Item {
    fn default() -> Self {
        NoItem
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
        Self::Holdable(DescriptionWithQualities::default())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DescriptionWithQualities {
    #[serde(flatten)]
    pub info: Description,
    #[serde(default)]
    pub attr: Vec<Quality>,
}

impl ItemDescribe for DescriptionWithQualities {}

impl Describe for DescriptionWithQualities {
    fn name(&self) -> String {
        self.info.name.clone()
    }

    fn display(&self) -> String {
        self.info.display.clone()
    }

    fn description(&self) -> String {
        self.info.description.clone()
    }

    fn handle(&self) -> Hook {
        self.info.handle.clone()
    }
}

impl Attribute<Quality> for DescriptionWithQualities {
    fn attr(&self) -> Vec<Quality> {
        self.attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.attr.push(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        let pos = self.attr.iter().position(|u| *u == q);
        if let Some(pos) = pos {
            self.attr.remove(pos);
        }
    }
}

impl DescriptionWithQualities {
    pub fn new(name: &str, description: Option<&str>, handle: Hook) -> Self {
        let description = description.unwrap_or_default().to_owned();
        let name = name.to_owned();
        let display = String::new();
        let attributes = Vec::new();

        Self {
            info: Description {
                name,
                description,
                handle,
                display,
            },
            attr: attributes,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct YamlItemList {
    inner: Vec<YamlItem>,
    #[serde(default)]
    info: DescriptionWithQualities,
}

impl Attribute<Quality> for YamlItemList {
    fn attr(&self) -> Vec<Quality> {
        self.info.attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.info.attr.push(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        let mut index = 0;
        for qual in self.info.attr.iter() {
            if *qual == q {
                break;
            }

            index += 1;
        }

        if index < self.info.attr.len() {
            self.info.attr.remove(index);
        }
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
            info: DescriptionWithQualities {
                info: Description {
                    name: "".to_string(),
                    display: "".to_string(),
                    description: "".to_string(),
                    handle: crate::handle![],
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
    fn safe_unwrap(&self) -> &DescriptionWithQualities {
        match self {
            YamlItem::Key(_, item)
            | YamlItem::Clothing(item)
            | YamlItem::Weapon(item)
            | YamlItem::Scenery(item)
            | YamlItem::Holdable(item)
            | YamlItem::Edible(item) => item,
            YamlItem::Container(i) => &i.info,
            YamlItem::Guard { info, .. } => &info,
        }
    }

    fn safe_unwrap_mut(&mut self) -> &mut DescriptionWithQualities {
        match self {
            YamlItem::Key(_, item)
            | YamlItem::Clothing(item)
            | YamlItem::Weapon(item)
            | YamlItem::Scenery(item)
            | YamlItem::Holdable(item)
            | YamlItem::Edible(item) => item,
            YamlItem::Container(i) => &mut i.info,
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

impl Attribute<Quality> for KeyType {
    fn attr(&self) -> Vec<Quality> {
        self.attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.attr.push(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        let mut index = 0;
        for qual in self.attr.iter() {
            if *qual == q {
                break;
            }

            index += 1;
        }

        if index < self.attr.len() {
            self.attr.remove(index);
        }
    }
}

impl From<DescriptionWithQualities> for KeyType {
    fn from(b: DescriptionWithQualities) -> Self {
        Self {
            info: b.info,
            attr: b.attr,
            key: 0,
        }
    }
}
