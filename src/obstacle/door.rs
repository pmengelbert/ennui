use crate::location::direction::MapDir;
use crate::location::Coord;
use crate::obstacle::key::Key;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Formatter};

use DoorState::{Locked, Open};

use std::option::NoneError;

pub type StateResult<T> = Result<(), T>;

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
