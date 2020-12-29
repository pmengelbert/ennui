pub mod error;
pub mod handle;
pub mod key;

use crate::game::MapDir;
use crate::item::handle::Handle;
use crate::item::key::{Key, KeyType};
use crate::map::door::{Guard, GuardState, RenaissanceGuard};
use serde::export::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut};
use std::mem::take;
use std::ops::{Deref, DerefMut};
use BasicItemKind::*;

pub trait Describe: Send + Sync + Debug + Attribute<Quality> {
    fn name(&self) -> &str;
    fn display(&self) -> &str;
    fn description(&self) -> &str;
    fn handle(&self) -> &Handle;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BasicItemKind {
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
    Container(GenericItemList),
    Key(u64, Description),
}

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
    fn is_a(&self, a: T) -> bool {
        self.attr().contains(&a)
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

impl Default for BasicItemKind {
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

pub trait Holder: Describe {
    type Kind;

    fn items(&self) -> &Self::Kind;
    fn items_mut(&mut self) -> &mut Self::Kind;
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
pub struct GenericItemList {
    inner: Vec<BasicItemKind>,
    info: Description,
    attributes: Vec<Quality>,
}

impl Attribute<Quality> for GenericItemList {
    fn attr(&self) -> &[Quality] {
        &self.info.attributes
    }
}

impl Deref for GenericItemList {
    type Target = Vec<BasicItemKind>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for GenericItemList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl AsRef<GenericItemList> for GenericItemList {
    fn as_ref(&self) -> &GenericItemList {
        self
    }
}

impl AsMut<GenericItemList> for GenericItemList {
    fn as_mut(&mut self) -> &mut GenericItemList {
        self
    }
}

impl Describe for GenericItemList {
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

impl GenericItemList {
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
            attributes: Vec::new(),
        }
    }
    pub fn get(&self, handle: &str) -> Option<&BasicItemKind> {
        self.iter().find(|i| i.handle() == handle)
    }

    pub fn get_owned<T: AsRef<str>>(&mut self, handle: T) -> Option<BasicItemKind> {
        let pos = self.iter().position(|i| i.handle() == handle.as_ref())?;
        Some(self.remove(pos))
    }
}

impl BasicItemKind {
    fn safe_unwrap(&self) -> &Description {
        match self {
            Key(_, item)
            | Clothing(item)
            | Weapon(item)
            | Scenery(item)
            | Holdable(item)
            | Edible(item) => &item,
            Container(i) => &i.info,
            BasicItemKind::Guard { info, .. } => &info,
        }
    }
}

impl Describe for BasicItemKind {
    fn name(&self) -> &str {
        &self.safe_unwrap()
            .name()
    }

    fn display(&self) -> &str {
        &self
            .safe_unwrap()
            .display()
    }

    fn description(&self) -> &str {
        &self
            .safe_unwrap()
            .description()
    }

    fn handle(&self) -> &Handle {
        &self.safe_unwrap().handle()
    }
}

impl Attribute<Quality> for BasicItemKind {
    fn attr(&self) -> &[Quality] {
        self.safe_unwrap().attr()
    }
}

pub trait ItemListTrait: Describe + Debug {
    type Kind: Debug;
    fn get(&self, handle: &str) -> Option<&Item>;
    fn get_mut(&mut self, handle: &str) -> Option<&mut Item>;
    fn get_owned(&mut self, handle: &str) -> Option<Item>;
    fn insert(&mut self, item: Item) -> Result<(), Item>;
    fn list(&self) -> &Self::Kind;

    fn transfer(
        &mut self,
        other: &mut dyn ItemListTrait<Kind = ItemList>,
        handle: &str,
    ) -> Result<String, String> {
        let handle = handle.as_ref();
        let item = match self.get_owned(handle) {
            Some(i) => i,
            None => return Err(handle.to_owned()),
        };

        let name = item.name().to_owned();
        if other.insert(item).is_err() {
            return Err("COULD NOT TRANSFER ITEM".into())
        };
        Ok(name)
    }
}

#[derive(Default, Debug)]
pub struct ItemList {
    inner: Vec<Item>,
    info: BasicItemKind,
}

impl Deref for ItemList {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ItemList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Describe for ItemList {
    fn name(&self) -> &str {
        self.info.name()
    }

    fn display(&self) -> &str {
        self.info.display()
    }

    fn description(&self) -> &str {
        self.info.description()
    }

    fn handle(&self) -> &Handle {
        self.info.handle()
    }

}

impl Attribute<Quality> for ItemList {
    fn attr(&self) -> &[Quality] {
        self.info.attr()
    }
}

impl ItemListTrait for ItemList {
    type Kind = ItemList;
    fn get(&self, handle: &str) -> Option<&Item> {
        self.iter()
            .find(|i| i.handle() == handle)
            .map(|i| i.borrow())
    }

    fn get_mut(&mut self, handle: &str) -> Option<&mut Item> {
        self.iter_mut()
            .find(|i| i.handle() == handle)
            .map(|i| i.borrow_mut())
    }

    fn get_owned(&mut self, handle: &str) -> Option<Item> {
        let pos = self.iter().position(|i| i.handle() == handle)?;
        Some(self.inner.remove(pos))
    }

    fn insert(&mut self, item: Item) -> Result<(), Item> {
        self.inner.push(item);
        Ok(())
    }

    fn list(&self) -> &Self::Kind {
        &self
    }
}

// impl IntoIterator for ItemList {
//     type Item = Item;
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self.list().into_iter()
//     }
// }

impl ItemList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            info: Default::default(),
        }
    }
}

impl From<GenericItemList> for ItemList {
    fn from(mut l: GenericItemList) -> Self {
        conv(&mut l)
    }
}

fn conv_desc(d: &mut Description, q: Quality) -> Box<dyn Describe> {
    let mut new = d.clone();
    new.attributes.push(q);
    Box::new(new)
}

fn conv(list: &mut GenericItemList) -> ItemList {
    let mut ret = ItemList::new();
    for i in &mut **list {
        println!("{:?}", i);
        let i = match i {
            Clothing(i) => Item::Clothing(conv_desc(i, Quality::Clothing)),
            Weapon(i) => Item::Weapon(conv_desc(i, Quality::Weapon)),
            Scenery(i) => Item::Scenery(conv_desc(i, Quality::Scenery)),
            Edible(i) => Item::Edible(conv_desc(i, Quality::Edible)),
            Holdable(i) => Item::Holdable(conv_desc(i, Quality::Holdable)),
            Container(ref mut listy) => Item::Container(Box::new(conv(listy))),
            BasicItemKind::Guard {
                dir, info, lock, ..
            } => {
                let mut g: RenaissanceGuard = take(info).into();
                g.lock = *lock;
                g.info.attributes.push(Quality::Container);
                Item::Guard(*dir, Box::new(g))
            }
            Key(n, item) => {
                let i = take(item);
                let mut k: KeyType = i.into();
                k.set_key(*n);
                k.add_quality(Quality::Key);
                Item::Key(Box::new(k))
            }
        };
        ret.push(i);
    }
    ret.info = Clothing(list.info.clone());
    ret
}

#[cfg(test)]
mod item_trait_test {
    use super::*;

    #[test]
    fn item_trait_test_1() {
        // let x: Vec<Box<ItemListTrait>> = Vec::new();
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
