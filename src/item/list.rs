use crate::error::CmdErr::ItemNotFound;
use crate::error::EnnuiError;
use crate::error::EnnuiError::{Fatal, Simple};
use crate::item::handle::Handle;
use crate::item::key::KeyType;
use crate::item::YamlItem::{Clothing, Container, Edible, Holdable, Key, Scenery, Weapon};
use crate::item::{Attribute, Describe, Description, Item, Quality, YamlItem, YamlItemList};
use crate::map::door::RenaissanceGuard;
use crate::text::message::MessageFormat;
use crate::text::Color::Green;
use std::fmt::Debug;
use std::mem::take;
use std::ops::{Deref, DerefMut};

pub trait Holder: Describe {
    type Kind;

    fn items(&self) -> &Self::Kind;
    fn items_mut(&mut self) -> &mut Self::Kind;
}

pub trait ListTrait: Describe + Debug {
    type Kind: Debug;
    fn get_item(&self, handle: &str) -> Option<&Item>;
    fn get_item_mut(&mut self, handle: &str) -> Option<&mut Item>;
    fn get_item_owned(&mut self, handle: &str) -> Result<Item, EnnuiError>;
    fn insert_item(&mut self, item: Item) -> Result<(), Item>;
    fn list(&self) -> &Self::Kind;

    fn transfer(
        &mut self,
        other: &mut dyn ListTrait<Kind = Self::Kind>,
        handle: &str,
    ) -> Result<String, EnnuiError> {
        let handle = handle.as_ref();
        let item = self.get_item_owned(handle)?;

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
    fn name(&self) -> String {
        self.info.name()
    }

    fn display(&self) -> String {
        self.info.display()
    }

    fn description(&self) -> String {
        self.info.description()
    }

    fn handle(&self) -> Handle {
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
    fn get_item(&self, handle: &str) -> Option<&Item> {
        self.iter().find(|i| i.handle() == handle)
    }

    fn get_item_mut(&mut self, handle: &str) -> Option<&mut Item> {
        self.iter_mut().find(|i| i.handle() == handle)
    }

    fn get_item_owned(&mut self, handle: &str) -> Result<Item, EnnuiError> {
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

impl ItemList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            info: Default::default(),
        }
    }

    pub fn get_owned(&mut self, handle: &str) -> Result<Item, EnnuiError> {
        let pos = match self.iter().position(|i| i.handle() == handle) {
            Some(i) => i,
            None => return Err(Simple(ItemNotFound)),
        };
        Ok(self.inner.remove(pos))
    }

    pub fn display(&self) -> String {
        let mut s = String::new();

        for item in self.iter() {
            s.push('\n');
            s.push_str(&item.display().color(Green));
        }

        s
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
    for i in &mut **list {
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
