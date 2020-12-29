use crate::item::handle::Handle;
use crate::item::{BasicItem, Describe};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait Key<T>: Describe + Debug {
    fn key(&self) -> T;
}

#[derive(Clone, Debug)]
pub struct SkeletonKey {
    pub handle: Handle,
}

impl Describe for SkeletonKey {
    fn name(&self) -> &str {
        "skeleton key"
    }

    fn display(&self) -> &str {
        "a rusted skeleton key"
    }

    fn description(&self) -> &str {
        "ok ok ok"
    }

    fn handle(&self) -> &Handle {
        &self.handle
    }

    fn is_container(&self) -> bool {
        false
    }
}

impl Key<u64> for SkeletonKey {
    fn key(&self) -> u64 {
        1
    }
}

pub trait KeyItem: Key<u64> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Codpiece(Handle);
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

impl From<BasicItem> for KeyType {
    fn from(i: BasicItem) -> Self {
        let BasicItem {
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
