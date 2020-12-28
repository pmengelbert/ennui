use crate::game::MapDir;
use crate::map::coord::Coord;
use crate::map::door::DoorState::{Locked, Open};
use crate::map::{Locate, StateResult};
use crate::player::Uuid;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::option::NoneError;
use crate::interpreter::CommandKind;

pub trait Keyhole<T, U>: ObstacleState<T>
where
    U: PartialEq<Self::Lock>,
{
    type Lock: PartialEq<U>;

    fn unlock(&mut self, new_state: T, key: Option<U>) -> StateResult<T>;
    fn is_locked(&self) -> bool;
}

pub trait ObstacleState<T> {
    fn state(&self) -> T;
}

pub trait Obstacle<T, U>: ObstacleState<U> {
    type Other: PartialEq<T>;

    fn dir(&self) -> MapDir;
    fn alt_dest(&self) -> Option<Coord>;
    fn change_state(&mut self, state: U, tool: Option<T>) -> Result<(), U>;

    fn destination<L>(&self, loc: L) -> Option<Coord>
    where
        L: Locate,
    {
        match self.alt_dest() {
            None => loc.loc().add(self.dir()),
            s => s,
        }
    }
}

// impl<T, U> Obstacle<T, DoorState> for Door<U>
// where
//     U: PartialEq<T> + Clone + Debug,
//     T: PartialEq<U>,
// {
//     type Other = U;
//
//     fn dir(&self) -> MapDir {
//         self.dir
//     }
//
//     fn alt_dest(&self) -> Option<Coord> {
//         self.alt_dest
//     }
// }

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
    MagicallySealed,
    PermaLocked,
    None,
}

impl From<std::option::NoneError> for DoorState {
    fn from(_: NoneError) -> Self {
        DoorState::None
    }
}

impl ObstacleState<DoorState> for DoorState {
    fn state(&self) -> DoorState {
        *self
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

impl Keyhole<DoorState, u64> for Door {
    type Lock = u64;

    fn unlock(&mut self, new_state: DoorState, key: Option<u64>) -> StateResult<DoorState> {
        if self.state == new_state {
            return Err(self.state);
        }

        match (self.keyhole.clone(), key) {
            (Some(h), Some(k)) if h == k => {
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
        match self.state {
            Open => false,
            _ => true,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DoorList(Vec<Door>);

impl Deref for DoorList {
    type Target = Vec<Door>;

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

    // #[test]
    // fn test_door_api_1() {
    //     let room = Room {
    //         name: "".to_string(),
    //         loc: Default::default(),
    //         description: "".to_string(),
    //         players: Default::default(),
    //         items: Default::default(),
    //         doors: HashMap::new(),
    //     };
    //     todo!()
    // }

    // #[test]
    // fn test_door_api_2() {
    //     let mut g = Game::new();
    //     let p = Player::new("billy").uuid();

    //     let res = Coord(0, 0).move_player(g, p, West);
    //     let res2 = Coord(0, 0).move_player(g, p, South);
    //     todo!()
    //     assert_eq!(res, Err(Closed("closed".into())));
    //     assert_eq!(res, Err(Locked("locked".into())));

    // }

    #[test]
    fn test_door_api_3() {
        let mut d: Door = Door {
            dir: North,
            state: DoorState::Locked,
            alt_dest: None,
            keyhole: Some(8),
        };

        let n: u64 = 8;
        let res = d.unlock(DoorState::Open, Some(n));
        assert_eq!(res, Ok(()));
        assert_eq!(d.state(), DoorState::Open);
        let res = d.unlock(Locked, Some(72));
        assert_eq!(res, Err(Locked));
        assert_eq!(d.state(), DoorState::Open);
        let res = d.unlock(Locked, None);
        assert_eq!(res, Err(Locked));
        assert_eq!(d.state(), DoorState::Open);
        let res = d.unlock(Open, None);
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

    fn unlock(&mut self, new_state: DoorState, key: Option<u64>) -> StateResult<DoorState> {
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

    fn unlock(&mut self, new_state: DoorState, key: Option<String>) -> StateResult<DoorState> {
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

#[cfg(test)]
mod wizard_test {
    use super::*;
    use crate::map::door::DoorState::Closed;

    #[test]
    fn wizard_test_1() {
        let mut w = Wizard {
            password: (CommandKind::Whisper, "until the dawn".to_owned()),
            state: Locked,
            f: Box::new((|state, key| {
                key - 1
            }))
        };

        assert_eq!(w.unlock(Open, Some(7)), Err(Locked));
        assert_eq!(w.unlock(Open, Some(2)), Ok(()));
        assert_eq!(w.state(), Open);
        let mut w2 = Wizard2 {
            password: (CommandKind::Whisper, "until the dawn".to_owned()),
            state: Locked,
            f: Box::new((|slf, state, key| {
                key == slf.password.1
            }))
        };
        assert_eq!(w2.unlock(Closed, Some("until the down".into())), Err(Locked));
        assert_eq!(w2.unlock(Closed, Some("until the dawn".into())), Ok(()));
        assert_eq!(w2.state(), Closed);

        let mut w3 = Wizard2 {
            password: (CommandKind::Whisper, "until the dawn".to_owned()),
            state: Locked,
            f: Box::new((|slf, state, key| {
                key == slf.password.1 && slf.password.0 == CommandKind::Whisper
            }))
        };
    }
}