use super::Description;
use crate::error::CmdErr::ItemNotFound;
use crate::error::EnnuiError;
use crate::error::EnnuiError::{Fatal, Simple};
use crate::gram_object::{Grabber, Hook};
use crate::item::YamlItem::{Clothing, Container, Edible, Holdable, Scenery, Weapon};
use crate::item::{
    Attribute, Describe, DescriptionWithQualities, Item, Quality, YamlItem, YamlItemList,
};
use crate::map::door::{GuardState, Lock, ObstacleState};
use crate::map::StateResult;
use crate::obstacle::key::{Key, KeyType};
use crate::text::message::MessageFormat;
use crate::text::Color::Green;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::mem::take;

use std::slice::{Iter, IterMut};

pub trait Holder: Describe {
    type Kind;

    fn items(&self) -> &Self::Kind;
    fn items_mut(&mut self) -> &mut Self::Kind;
}

pub trait ListTrait: Describe + Debug {
    type Kind: Debug;
    fn get_item(&self, handle: Grabber) -> Option<&Item>;
    fn get_item_mut(&mut self, handle: Grabber) -> Option<&mut Item>;
    fn get_item_owned(&mut self, handle: Grabber) -> Result<Item, EnnuiError>;
    fn insert_item(&mut self, item: Item) -> Result<(), Item>;
    fn list(&self) -> &Self::Kind;

    fn transfer(
        &mut self,
        other: &mut dyn ListTrait<Kind = Self::Kind>,
        handle: &str,
    ) -> Result<String, EnnuiError> {
        let item = self.get_item_owned(handle.into())?;

        let name = item.name();
        if other.insert_item(item).is_err() {
            return Err(Fatal("COULD NOT TRANSFER ITEM".into()));
        };
        Ok(name)
    }
}

#[derive(Default, Debug)]
pub struct ItemList {
    inner: Vec<Item>,
    info: super::DescriptionWithQualities,
}

impl Describe for ItemList {
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

impl Attribute<Quality> for ItemList {
    fn attr(&self) -> Vec<Quality> {
        self.info.attr()
    }

    fn set_attr(&mut self, q: Quality) {
        self.info.set_attr(q)
    }

    fn unset_attr(&mut self, q: Quality) {
        self.info.unset_attr(q);
    }
}

impl ListTrait for ItemList {
    type Kind = ItemList;
    fn get_item(&self, handle: Grabber) -> Option<&Item> {
        self.iter()
            .filter(|i| i.handle() == handle.handle)
            .nth(handle.index)
    }

    fn get_item_mut(&mut self, handle: Grabber) -> Option<&mut Item> {
        self.iter_mut()
            .filter(|i| i.handle() == handle.handle)
            .nth(handle.index)
    }

    fn get_item_owned(&mut self, handle: Grabber) -> Result<Item, EnnuiError> {
        self.get_owned(handle)
    }

    fn insert_item(&mut self, item: Item) -> Result<(), Item> {
        self.inner.push(item);
        Ok(())
    }

    fn list(&self) -> &Self::Kind {
        &self
    }
}

pub trait ItemListTrout {
    fn get_owned(&mut self, handle: Grabber) -> Result<Item, EnnuiError>;
    fn display_items(&self) -> String;
    fn iter(&self) -> Iter<Item>;
    fn iter_mut(&mut self) -> IterMut<Item>;
    fn len(&self) -> usize;
    fn push(&mut self, i: Item);
}

impl ItemList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            info: Default::default(),
        }
    }

    pub fn new_with_info(info: super::DescriptionWithQualities) -> Self {
        Self {
            inner: vec![],
            info,
        }
    }
}

impl ItemListTrout for ItemList {
    fn get_owned(&mut self, handle: Grabber) -> Result<Item, EnnuiError> {
        let Grabber { handle, index } = handle;

        let pos = self
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                if item.handle() == handle {
                    Some(idx)
                } else {
                    None
                }
            })
            .nth(index)
            .ok_or_else(|| Simple(ItemNotFound))?;
        Ok(self.inner.remove(pos))
    }

    fn display_items(&self) -> String {
        let mut s = String::new();

        for item in self.iter() {
            s.push('\n');
            s.push_str(&item.display().color(Green));
        }

        s
    }

    fn iter(&self) -> Iter<Item> {
        self.inner.iter()
    }

    fn iter_mut(&mut self) -> IterMut<Item> {
        self.inner.iter_mut()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn push(&mut self, i: Item) {
        self.inner.push(i);
    }
}

impl From<YamlItemList> for ItemList {
    fn from(mut l: YamlItemList) -> Self {
        conv(&mut l)
    }
}

fn conv_desc(d: &mut DescriptionWithQualities, q: Quality) -> Box<dyn super::ItemDescribe> {
    let mut new = d.clone();
    d.set_attr(q);
    Box::new(new)
}

fn conv(list: &mut YamlItemList) -> ItemList {
    let mut ret = ItemList::new();
    for i in list.inner.iter_mut() {
        let i = match i {
            Clothing(i) => Item::Clothing(conv_desc(i, Quality::Clothing)),
            Weapon(i) => Item::Weapon(conv_desc(i, Quality::Weapon)),
            Scenery(i) => Item::Scenery(conv_desc(i, Quality::Scenery)),
            Edible(i) => Item::Edible(conv_desc(i, Quality::Edible)),
            Holdable(i) => Item::Holdable(conv_desc(i, Quality::Holdable)),
            Container(ref mut listy) => Item::Container(Box::new(conv(listy))),
            YamlItem::Guard {
                dir, info, lock, ..
            } => {
                let mut g: RenaissanceGuard = take(info).into();
                g.lock = *lock;
                g.set_attr(Quality::Container);
                Item::Guard(*dir, Box::new(g))
            }
            YamlItem::Key(n, item) => {
                let i = take(item);
                let mut k: KeyType = i.into();
                k.set_key(*n);
                k.add_quality(Quality::Key);
                Item::Key(Box::new(k))
            }
        };
        ret.push(i);
    }
    ret.info = list.info.clone();
    ret
}

pub trait Guard: Lock<GuardState> + ListTrait<Kind = ItemList> {}
impl Guard for RenaissanceGuard {}

impl ListTrait for RenaissanceGuard {
    type Kind = ItemList;

    fn get_item(&self, handle: Grabber) -> Option<&Item> {
        self.items.get_item(handle)
    }

    fn get_item_mut(&mut self, handle: Grabber) -> Option<&mut Item> {
        self.items.get_item_mut(handle)
    }

    fn get_item_owned(&mut self, handle: Grabber) -> Result<Item, EnnuiError> {
        self.items.get_item_owned(handle)
    }

    fn insert_item(&mut self, item: Item) -> Result<(), Item> {
        match &item {
            Item::Key(k) => match self.unlock(GuardState::Open, Some(&**k)) {
                Ok(()) => Ok(()),
                Err(_) => Err(item),
            },
            _ => Err(item),
        }
    }

    fn list(&self) -> &Self::Kind {
        self.items.list()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RenaissanceGuard {
    #[serde(skip_serializing, skip_deserializing)]
    pub items: ItemList,
    #[serde(default)]
    pub state: GuardState,
    pub lock: u64,
    pub info: Description,
    #[serde(default)]
    pub attr: Vec<Quality>,
}

impl crate::item::GuardDescribe for RenaissanceGuard {}

impl Clone for RenaissanceGuard {
    fn clone(&self) -> Self {
        Self {
            info: self.info.clone(),
            items: ItemList::new(),
            state: self.state,
            lock: self.lock,
            attr: self.attr.clone(),
        }
    }
}

impl Describe for RenaissanceGuard {
    fn name(&self) -> String {
        self.info.name()
    }

    fn display(&self) -> String {
        self.info.display()
    }

    fn description(&self) -> String {
        match self.state() {
            GuardState::Closed => {
                self.info.description()
            }
            GuardState::Open => {
                "He seems happy as a clam, and tells you over and over how grateful he is to have warm genitals.".to_owned()
            }
        }
    }

    fn handle(&self) -> Hook {
        self.info.handle()
    }
}

impl Attribute<Quality> for RenaissanceGuard {
    fn attr(&self) -> Vec<Quality> {
        self.attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.set_attr(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        self.unset_attr(q);
    }
}

impl ObstacleState<GuardState> for RenaissanceGuard {
    fn state(&self) -> GuardState {
        self.state
    }
}

impl Lock<GuardState> for RenaissanceGuard {
    type Lock = u64;

    fn unlock(
        &mut self,
        new_state: GuardState,
        key: Option<&dyn Key<Self::Lock>>,
    ) -> StateResult<GuardState> {
        if let GuardState::Open = new_state {
            match key {
                Some(k) if k.key() == self.lock => {
                    self.state = new_state;
                    Ok(())
                }
                Some(_k) => Err(self.state()),
                _ => Err(self.state()),
            }
        } else {
            Err(self.state())
        }
    }

    fn is_locked(&self) -> bool {
        self.state == GuardState::Closed
    }
}

impl From<DescriptionWithQualities> for RenaissanceGuard {
    fn from(b: DescriptionWithQualities) -> Self {
        Self {
            info: b.info,
            attr: b.attr,
            ..Self::default()
        }
    }
}
