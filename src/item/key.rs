use crate::attribute::Attribute;
use crate::describe::{Describe, Description};
use crate::gram_object::Hook;
use crate::item::Quality;
use std::fmt::Debug;

pub trait Key<T>: Describe + Debug + Attribute<Quality> {
    fn key(&self) -> T;
}

#[derive(Clone, Debug)]
pub struct KeyType {
    info: Description,
    key: u64,
    attr: Vec<super::Quality>,
}

impl KeyType {
    pub fn set_key(&mut self, key: u64) {
        self.key = key
    }
}

impl Describe for KeyType {
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

impl Attribute<Quality> for KeyType {
    fn attr(&self) -> Vec<Quality> {
        self.attr.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.attr.set_attr(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        self.attr.unset_attr(q);
    }
}

impl From<super::Item2> for KeyType {
    fn from(b: super::Item2) -> Self {
        Self {
            info: Description {
                name: b.name(),
                display: b.display(),
                description: b.description(),
                handle: b.handle(),
            },
            key: 0,
            attr: vec![super::Quality::Key],
        }
    }
}

impl Key<u64> for KeyType {
    fn key(&self) -> u64 {
        self.key
    }
}
