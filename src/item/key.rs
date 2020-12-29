use crate::item::handle::Handle;
use crate::item::{Describe, Description};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait Key<T>: Describe + Debug {
    fn key(&self) -> T;
}

#[derive(Clone, Debug)]
pub struct SkeletonKey {
    info: Description,
    key: u64,
}

impl SkeletonKey {
    pub fn set_key(&mut self, key: u64) {
        self.key = key
    }
}

impl Describe for SkeletonKey {
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

    fn is_container(&self) -> bool {
        false
    }
}

impl From<Description> for SkeletonKey {
    fn from(b: Description) -> Self {
        Self { info: b, key: 0 }
    }
}

impl Key<u64> for SkeletonKey {
    fn key(&self) -> u64 {
        self.key
    }
}

pub trait KeyItem: Key<u64> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyType {
    name: String,
    display: String,
    description: String,
    handle: Handle,
    pub key: u64,
}

impl KeyItem for KeyType {}

impl Describe for KeyType {
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

    fn is_container(&self) -> bool {
        true
    }
}

impl From<Description> for KeyType {
    fn from(i: Description) -> Self {
        let Description {
            name,
            display,
            description,
            handle,
        } = i;

        Self {
            name,
            display,
            description,
            handle,
            key: 0,
        }
    }
}

// impl Describe for Codpiece {
//     fn name(&self) -> &str {
//         "codpiece"
//     }
//
//     fn display(&self) -> &str {
//         "A tattered old codpiece is here, mocking you."
//     }
//
//     fn description(&self) -> &str {
//         "It's very ornate, but it's still very much a codpiece. You see no need for it, and yet \
//         you simply can't resist the urge to put it on. You can't rationalize its power over you, and you \
//         hang your head, ashamed."
//     }
//
//     fn handle(&self) -> &Handle {
//         &self.0
//     }
//
//     fn is_container(&self) -> bool {
//         false
//     }
// }

impl Key<u64> for KeyType {
    fn key(&self) -> u64 {
        self.key
    }
}
