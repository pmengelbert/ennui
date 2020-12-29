use crate::item::handle::Handle;
use crate::item::{Describe, Description, Quality, Attribute};
use std::fmt::Debug;

pub trait Key<T>: Describe + Debug {
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
        self.info.attributes.push(q);
    }
}

impl Describe for KeyType {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn display(&self) -> &str {
        &self.info.display
    }

    fn description(&self) -> &str {
        &self.info.description
    }

    fn handle(&self) -> &Handle {
        &self.info.handle
    }
}

impl Attribute<Quality> for KeyType {
    fn attr(&self) -> &[Quality] {
        &self.info.attributes
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

