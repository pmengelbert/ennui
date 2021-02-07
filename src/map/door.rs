use crate::error::EnnuiError;
use crate::item::handle::{Grabber, Hook};
use crate::item::key::Key;
use crate::item::list::{ItemList, ListTrait};
use crate::item::{Attribute, Describe, Description, Item, Quality};
use crate::map::coord::Coord;
use crate::map::direction::MapDir;
use crate::map::door::DoorState::{Locked, Open};
use crate::map::StateResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Formatter};

use std::option::NoneError;

pub trait Lock<T>: ObstacleState<T> {
    type Lock;

    fn unlock(&mut self, new_state: T, key: Option<&dyn Key<Self::Lock>>) -> StateResult<T>;
    fn is_locked(&self) -> bool;
}

impl Lock<DoorState> for Door {
    type Lock = u64;

    fn unlock(
        &mut self,
        new_state: DoorState,
        key: Option<&dyn Key<Self::Lock>>,
    ) -> StateResult<DoorState> {
        if self.state == new_state {
            return Err(self.state.clone());
        }

        match (new_state.clone(), self.state()) {
            (Open, DoorState::Closed) => {
                self.state = new_state;
                return Ok(());
            }
            (DoorState::Closed, Open) => return Err(self.state()),
            _ => (),
        }

        match (self.keyhole, key) {
            (Some(h), Some(k)) if h == k.key() => {
                self.state = new_state;
                Ok(())
            }
            (None, _) => {
                self.state = new_state;
                Ok(())
            }
            _ => Err(Locked),
        }
    }

    fn is_locked(&self) -> bool {
        !matches!(self.state(), DoorState::Open | DoorState::Closed)
    }
}

pub trait ObstacleState<T> {
    fn state(&self) -> T;
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RenaissanceGuard {
    #[serde(skip_serializing, skip_deserializing)]
    items: ItemList,
    #[serde(default)]
    state: GuardState,
    pub lock: u64,
    pub info: Description,
}

impl Clone for RenaissanceGuard {
    fn clone(&self) -> Self {
        Self {
            info: self.info.clone(),
            items: ItemList::new(),
            state: self.state,
            lock: self.lock,
        }
    }
}

impl From<Description> for RenaissanceGuard {
    fn from(b: Description) -> Self {
        Self {
            info: b,
            ..Self::default()
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
        self.info.attributes.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.info.set_attr(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        self.info.unset_attr(q);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum GuardState {
    Open,
    Closed,
}

impl Default for GuardState {
    fn default() -> Self {
        GuardState::Closed
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

pub trait Guard: Lock<GuardState> + ListTrait<Kind = ItemList> {}

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

impl Guard for RenaissanceGuard {}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
    MagicallySealed,
    PermaLocked,
    Guarded(String),
    None,
}

impl From<std::option::NoneError> for DoorState {
    fn from(_: NoneError) -> Self {
        DoorState::None
    }
}

impl ObstacleState<DoorState> for DoorState {
    fn state(&self) -> DoorState {
        self.clone()
    }
}

impl Default for DoorState {
    fn default() -> Self {
        Open
    }
}

impl std::fmt::Display for DoorState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DoorState {}

#[derive(Eq, PartialEq, Serialize, Deserialize, Default, Debug, Clone)]
pub struct Door {
    dir: MapDir,
    state: DoorState,
    alt_dest: Option<Coord>,
    // the key's id must match the Keyhole's id
    keyhole: Option<u64>,
}

impl ObstacleState<DoorState> for Door {
    fn state(&self) -> DoorState {
        self.state.clone()
    }
}

pub type DoorList = HashMap<MapDir, Door>;
