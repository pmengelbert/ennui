use crate::attribute::{Attribute, Quality};
use crate::describe::{Describe, Description};
use crate::hook::Hook;
use std::fmt::Debug;

pub trait Key<T>: Describe + Debug + Attribute<Quality> {
    fn key(&self) -> T;
}

#[derive(Clone, Debug)]
pub struct KeyType {
    pub info: Description,
    pub attr: Vec<Quality>,
    pub key: u64,
}

impl KeyType {
    pub fn set_key(&mut self, key: u64) {
        self.key = key
    }

    pub fn add_quality(&mut self, q: Quality) {
        self.attr.push(q);
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

impl Key<u64> for KeyType {
    fn key(&self) -> u64 {
        self.key
    }
}
