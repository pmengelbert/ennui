use crate::error::CmdErr::ItemNotFound;
use crate::error::EnnuiError;
use crate::error::EnnuiError::{Fatal, Simple};
use crate::item::handle::{Grabber, Hook};
use crate::item::key::KeyType;
use crate::item::YamlItem::{Clothing, Container, Edible, Holdable, Key, Scenery, Weapon};
use crate::item::{Attribute, Describe, Description, Item, Quality, YamlItem, YamlItemList};
use crate::map::door::RenaissanceGuard;
use crate::text::message::MessageFormat;
use crate::text::Color::Green;
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
    info: YamlItem,
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

    pub fn new_with_info(info: Description) -> Self {
        Self {
            inner: vec![],
            info: YamlItem::Holdable(info),
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

fn conv_desc(d: &mut Description, q: Quality) -> Box<dyn Describe> {
    let mut new = d.clone();
    new.attributes.push(q);
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
