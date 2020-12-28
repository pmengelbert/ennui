pub mod error;
pub mod handle;
pub mod key;

use crate::item::handle::Handle;
use serde::export::fmt::Debug;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use BasicItemKind::*;
use std::borrow::{Borrow, Cow};
use crate::item::key::Key;
use crate::map::Room;

pub trait ItemTrait : Send + Sync + Debug {
    fn name(&self) -> &str;
    fn display(&self) -> &str;
    fn description(&self) -> &str;
    fn handle(&self) -> &Handle;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BasicItemKind {
    Clothing(BasicItem),
    Weapon(BasicItem),
    Scenery(BasicItem),
    Edible(BasicItem),
    Holdable(BasicItem),
    Container(GenericItemList)
}

#[derive(Debug)]
pub enum Item {
    Clothing(Box<dyn ItemTrait>),
    Weapon(Box<dyn ItemTrait>),
    Scenery(Box<dyn ItemTrait>),
    Edible(Box<dyn ItemTrait>),
    Holdable(Box<dyn ItemTrait>),
    Container(Box<dyn ItemListTrait<Other = ItemList2>>),
    Key(Box<dyn Key<u64>>)
}

impl ItemTrait for Item {
    fn name(&self) -> &str {
        use Item::*;
        match self {
            Clothing(i)
            | Weapon(i)
            | Scenery(i)
            | Edible(i)
            | Holdable(i) => i.name(),
            Container(i) => i.name(),
            Key(i) => i.name(),
        }
    }

    fn display(&self) -> &str {
        use Item::*;
        match self {
            Clothing(i)
            | Weapon(i)
            | Scenery(i)
            | Edible(i)
            | Holdable(i) => i.display(),
            Container(i) => i.display(),
            Key(i) => i.display(),
        }
    }

    fn description(&self) -> &str {
        use Item::*;
        match self {
            Clothing(i)
            | Weapon(i)
            | Scenery(i)
            | Edible(i)
            | Holdable(i) => i.description(),
            Container(i) => i.description(),
            Key(i) => i.description(),
        }
    }

    fn handle(&self) -> &Handle {
        use Item::*;
        match self {
            Clothing(i)
            | Weapon(i)
            | Scenery(i)
            | Edible(i)
            | Holdable(i) => i.handle(),
            Container(i) => i.handle(),
            Key(i) => i.handle(),
        }
    }
}

impl Default for BasicItemKind {
    fn default() -> Self {
        Holdable(BasicItem::default())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BasicItem {
    name: String,
    display: String,
    description: String,
    handle: Handle,
}

impl ItemTrait for BasicItem {
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

pub trait Holder : ItemTrait {
    type Kind;

    fn items(&self) -> &Self::Kind;
    fn items_mut(&mut self) -> &mut Self::Kind;

    // fn transfer(&mut self, other: &mut ItemListTrait<Other = ItemList2>, handle: &str) -> Result<String, String>
    // where
    // {
    //     let handle = handle.as_ref();
    //     let item = match self.items_mut().get_owned(handle) {
    //         Some(i) => i,
    //         None => return Err(handle.to_owned()),
    //     };

    //     let name = item.name().to_owned();
    //     other.items_mut().insert(item);
    //     Ok(name)
    // }
}

// impl<T> Holder<ItemList2, Cow<'static, str>> for T
// where
//     T: AsRef<ItemList2> + AsMut<ItemList2> + ItemTrait,
// {
//     fn items(&self) -> &ItemList2 {
//         self.as_ref()
//     }
//
//     fn items_mut(&mut self) -> &mut ItemList2 {
//         self.as_mut()
//     }
// }


impl BasicItem {
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GenericItemList{
    inner: Vec<BasicItemKind>,
    name: String,
    display: String,
    description: String,
    handle: Handle,
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

impl GenericItemList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            name: "".to_string(),
            display: "".to_string(),
            description: "".to_string(),
            handle: Default::default()
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
    fn safe_unwrap(&self) -> Option<&BasicItem> {
        match self {
            Clothing(item) | Weapon(item) | Scenery(item) | Holdable(item) | Edible(item) => Some(&item),
            Container(_) => None,
        }
    }
}

impl ItemTrait for BasicItemKind {
    fn name(&self) -> &str {
        self.safe_unwrap().map(|i| i.name.as_str()).unwrap_or_default()
    }

    fn display(&self) -> &str {
        &self.safe_unwrap().map(|i| i.display.as_str()).unwrap_or_default()
    }

    fn description(&self) -> &str {
        &self.safe_unwrap().map(|i| i.description.as_str()).unwrap_or_default()
    }

    fn handle(&self) -> &Handle {
        &self.safe_unwrap().map(|i| i.handle()).as_ref().unwrap()
    }
}

pub trait ItemListTrait : ItemTrait + Debug {
    type Other: Debug;
    fn get(&self, handle: &str) -> Option<&Item>;
    fn get_owned(&mut self, handle: &str) -> Option<Item>;
    fn insert(&mut self, item: Item);

    fn transfer(&mut self, other: &mut ItemListTrait<Other = ItemList2>, handle: &str) -> Result<String, String> {
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

#[derive(Default, Debug)]
pub struct ItemList2 {
    inner: Vec<Item>,
    info: BasicItemKind,
}

impl Deref for ItemList2 {
    type Target = Vec<Item>;

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

    fn handle(&self) -> &Handle {
        self.info.handle()
    }
}


impl ItemListTrait for ItemList2 {
    type Other = ItemList2;
    fn get(&self, handle: &str) -> Option<&Item> {
        self.iter().find(|i| i.handle() == handle).map(|i| i.borrow())
    }

    fn get_owned(&mut self, handle: &str) -> Option<Item> {
        let pos = self.iter().position(|i| i.handle() == handle)?;
        Some(self.inner.remove(pos))
    }

    fn insert(&mut self, item: Item) {
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

impl From<GenericItemList> for ItemList2 {
    fn from(l: GenericItemList) -> Self {
        println!("{:?}", l);
        let mut v: Vec<Item> = Vec::new();
        for i in &*l {
            let i = match i {
               Clothing(i) => Item::Clothing(Box::new(i.clone())),
                Weapon(i) => Item::Weapon(Box::new(i.clone())),
                Scenery(i) => Item::Scenery(Box::new(i.clone())),
                Edible(i) => Item::Edible(Box::new(i.clone())),
                Holdable(i) => Item::Holdable(Box::new(i.clone())),
                Container(listy) => Item::Container(Box::new(conv(listy))),
            };
            v.push(i);
        }
        let ret = ItemList2 {
            inner: v,
            info: Clothing(BasicItem {
                name: l.name.to_owned(),
                display: l.display.to_owned(),
                description: l.description.to_owned(),
                handle: l.handle.to_owned(),
            })
        };
        println!("{:?}", ret);
        ret
    }
}

fn conv(list: &GenericItemList) -> ItemList2 {
    let mut ret = ItemList2::new();
    for i in &**list {
        println!("{:?}", i);
        let i = match i {
            Clothing(i) => Item::Clothing(Box::new(i.clone())),
            Weapon(i) => Item::Weapon(Box::new(i.clone())),
            Scenery(i) => Item::Scenery(Box::new(i.clone())),
            Edible(i) => Item::Edible(Box::new(i.clone())),
            Holdable(i) => Item::Holdable(Box::new(i.clone())),
            Container(listy) => Item::Container(Box::new(conv(listy))),
        };
        ret.push(i);
    }
    ret.info = Clothing(BasicItem {
        name: list.name.to_owned(),
        display: list.display.to_owned(),
        description: list.description.to_owned(),
        handle: list.handle.to_owned(),
    });
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

impl AsRef<ItemList2> for ItemList2 {
    fn as_ref(&self) -> &ItemList2 {
        self
    }
}

impl AsMut<ItemList2> for ItemList2 {
    fn as_mut(&mut self) -> &mut ItemList2 {
        self
    }
}

// impl Holder for ItemList2 {
//     type Kind = Self;
//
//     fn items(&self) -> &ItemList2 {
//         self
//     }
//
//     fn items_mut(&mut self) -> &mut ItemList2 {
//         &mut self
//     }
// }