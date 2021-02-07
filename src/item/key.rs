use crate::gram_object::Hook;
use crate::item::{Attribute, Describe, Description, Quality};
use std::fmt::Debug;

pub trait Key<T>: Describe + Debug + Attribute<Quality> {
    fn key(&self) -> T;
}

#[derive(Clone, Debug)]
pub struct KeyType {
    info: Description,
    key: u64,
}

impl KeyType {
    pub fn set_key(&mut self, key: u64) {
        self.key = key
    }

    pub fn add_quality(&mut self, q: Quality) {
        self.info.attr.push(q);
    }
}

impl Describe for KeyType {
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

impl Attribute<Quality> for KeyType {
    fn attr(&self) -> Vec<Quality> {
        self.info.attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.info.set_attr(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        self.info.unset_attr(q);
    }
}

impl From<Description> for KeyType {
    fn from(b: Description) -> Self {
        Self { info: b, key: 0 }
    }
}

impl Key<u64> for KeyType {
    fn key(&self) -> u64 {
        self.key
    }
}
