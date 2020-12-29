use crate::game::MapDir;
use crate::item::handle::Handle;
use crate::item::key::{Key, KeyType};
use crate::item::{Describe, Item, ItemList, ItemListTrait};
use crate::map::coord::Coord;
use crate::map::door::DoorState::{Locked, Open};
use crate::map::{Locate, StateResult};
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
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
        match self.state() {
            DoorState::Open | DoorState::Closed => false,
            _ => true,
        }
    }
}

pub trait ObstacleState<T> {
    fn state(&self) -> T;
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RenaissanceGuard {
    handle: Handle,
    #[serde(skip_serializing, skip_deserializing)]
    items: ItemList,
    state: GuardState,
    lock: u64,
}

impl Clone for RenaissanceGuard {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            items: ItemList::new(),
            state: self.state,
            lock: self.lock,
        }
    }
}

impl Describe for RenaissanceGuard {
    fn name(&self) -> &str {
        "Renaissance guard"
    }

    fn display(&self) -> &str {
        "A man in a floppy hat is here, shooting the breeze."
    }

    fn description(&self) -> &str {
        match self.state() {
            GuardState::Closed => {
                "The man seems to have a certain poise. He looks as though he's escaped from a Renaissance \
                faire. Your illusions of his poise vanish rapidly as he begins screaming, \"MY GENITALS ARE COLD, \
                YOU OUTRAGEOUS SCOUNDREL!! BRING ME SOMETHING FOR TO COVER MY GENITALS!\". You start to back away \
                but then, why not help the guy? Maybe he'll help you back."
            }
            GuardState::Open => {
                "He seems happy as a clam, and tells you over and over how grateful he is to have warm genitals."
            }
        }
    }

    fn handle(&self) -> &Handle {
        &self.handle
    }

    fn is_container(&self) -> bool {
        true
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
                    println!("guard state: {:?}", self.state());
                    Ok(())
                }
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

pub trait Guard: Lock<GuardState> + ItemListTrait<Kind = ItemList> {}

impl ItemListTrait for RenaissanceGuard {
    type Kind = ItemList;

    fn get(&self, handle: &str) -> Option<&Item> {
        self.items.get(handle)
    }

    fn get_mut(&mut self, handle: &str) -> Option<&mut Item> {
        self.items.get_mut(handle)
    }

    fn get_owned(&mut self, handle: &str) -> Option<Item> {
        self.items.get_owned(handle)
    }

    fn insert(&mut self, item: Item) -> Result<(), ()> {
        match item {
            Item::Key(k) => match self.unlock(GuardState::Open, Some(&*k)) {
                Ok(()) => Ok(()),
                Err(state) => Err(()),
            },
            _ => Err(()),
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
    // name
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

#[repr(transparent)]
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DoorList(pub HashMap<MapDir, Door>);

impl Deref for DoorList {
    type Target = HashMap<MapDir, Door>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DoorList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test_doors {
    use super::*;
    use crate::game::Game;
    use crate::game::MapDir::{North, South, West};
    use crate::map::door::DoorState::{Closed, Locked};
    use crate::map::{Locate, Room};
    use crate::player::Player;
    use std::collections::HashMap;

    #[test]
    fn test_door_api_3() {
        let mut d: Door = Door {
            dir: North,
            state: DoorState::Locked,
            alt_dest: None,
            keyhole: Some(8),
        };

        let n: u64 = 8;
        let res = d.unlockTODO(DoorState::Open, Some(n));
        assert_eq!(res, Ok(()));
        assert_eq!(d.state(), DoorState::Open);
        let res = d.unlockTODO(Locked, Some(72));
        assert_eq!(res, Err(Locked));
        assert_eq!(d.state(), DoorState::Open);
        let res = d.unlockTODO(Locked, None);
        assert_eq!(res, Err(Locked));
        assert_eq!(d.state(), DoorState::Open);
        let res = d.unlockTODO(Open, None);
        assert_eq!(res, Err(Open));
    }

    #[test]
    fn door_yaml() {
        let mut h = HashMap::new();
        let mut d: Door = Door {
            dir: North,
            state: DoorState::Locked,
            alt_dest: Some(Coord(7, 7)),
            keyhole: Some(8),
        };
        let mut e: Door = Door {
            dir: West,
            state: DoorState::Locked,
            alt_dest: Some(Coord(7, 9)),
            keyhole: Some(4),
        };
        h.insert(d.dir, d);
        h.insert(e.dir, e);
        let x = serde_yaml::to_vec(&h).unwrap();
        std::fs::write("/tmp/doormap.yaml", x);
    }
}

#[cfg(test)]
mod wizard_test {
    use super::*;
    use crate::interpreter::CommandKind;
    use crate::map::door::DoorState::Closed;

    struct Wizard {
        state: DoorState,
        password: (CommandKind, String),
        f: Box<Fn(DoorState, u64) -> u64>,
    }

    struct Wizard2 {
        state: DoorState,
        password: (CommandKind, String),
        f: Box<Fn(&Self, DoorState, String) -> bool>,
    }

    impl Keyhole<DoorState, u64> for Wizard {
        type Lock = u64;

        fn unlockTODO(&mut self, new_state: DoorState, key: Option<u64>) -> StateResult<DoorState> {
            if let Some(n) = key {
                return if (*self.f)(new_state, n) == 1 {
                    self.state = new_state;
                    Ok(())
                } else {
                    Err(Locked)
                };
            }

            Err(self.state())
        }

        fn is_locked(&self) -> bool {
            true
        }
    }

    impl ObstacleState<DoorState> for Wizard {
        fn state(&self) -> DoorState {
            self.state
        }
    }

    impl ObstacleState<DoorState> for Wizard2 {
        fn state(&self) -> DoorState {
            self.state
        }
    }

    impl Keyhole<DoorState, String> for Wizard2 {
        type Lock = String;

        fn unlockTODO(
            &mut self,
            new_state: DoorState,
            key: Option<String>,
        ) -> StateResult<DoorState> {
            if let Some(key) = key {
                return if (*self.f)(self, new_state, key) {
                    self.state = new_state;
                    Ok(())
                } else {
                    Err(self.state())
                };
            }
            Err(self.state())
        }

        fn is_locked(&self) -> bool {
            unimplemented!()
        }
    }

    #[test]
    fn wizard_test_1() {
        let mut w = Wizard {
            password: (CommandKind::Whisper, "until the dawn".to_owned()),
            state: Locked,
            f: Box::new((|state, key| key - 1)),
        };

        assert_eq!(w.unlockTODO(Open, Some(7)), Err(Locked));
        assert_eq!(w.unlockTODO(Open, Some(2)), Ok(()));
        assert_eq!(w.state(), Open);
        let mut w2 = Wizard2 {
            password: (CommandKind::Whisper, "until the dawn".to_owned()),
            state: Locked,
            f: Box::new((|slf, state, key| key == slf.password.1)),
        };
        assert_eq!(
            w2.unlockTODO(Closed, Some("until the down".into())),
            Err(Locked)
        );
        assert_eq!(w2.unlockTODO(Closed, Some("until the dawn".into())), Ok(()));
        assert_eq!(w2.state(), Closed);

        let mut w3 = Wizard2 {
            password: (CommandKind::Whisper, "until the dawn".to_owned()),
            state: Locked,
            f: Box::new(
                (|slf, state, key| key == slf.password.1 && slf.password.0 == CommandKind::Whisper),
            ),
        };
    }
}
