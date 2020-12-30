use crate::item::handle::Handle;
use crate::item::YamlItem::{Clothing, Weapon, Scenery, Edible, Holdable, Container, Key};
use crate::map::door::RenaissanceGuard;
use crate::item::key::KeyType;
use std::ops::{Deref, DerefMut};
use std::fmt::Debug;
use crate::item::{Describe, Item, YamlItem, Attribute, Quality, YamlItemList, Description};
use std::mem::take;

pub trait Holder: Describe {
    type Kind;

    fn items(&self) -> &Self::Kind;
    fn items_mut(&mut self) -> &mut Self::Kind;
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
            return Err("COULD NOT TRANSFER ITEM".into());
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
    }

    fn get_mut(&mut self, handle: &str) -> Option<&mut Item> {
        self.iter_mut()
            .find(|i| i.handle() == handle)
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

impl ItemList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            info: Default::default(),
        }
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
        println!("{:?}", i);
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
