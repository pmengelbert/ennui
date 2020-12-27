use crate::game::MapDir;
use crate::map::coord::Coord;
use crate::map::door::DoorState::{Locked, Open};
use crate::map::Locate;
use crate::player::Uuid;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub trait ObstacleState<T> {
    fn state(&self) -> T;
    fn is_blocked(&self) -> bool;
}

pub trait Obstacle<T, U> : ObstacleState<U> {
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

impl<T, U> Obstacle<T, DoorState> for Door<U>
where
    U: PartialEq<T> + Clone + Debug,
    T: PartialEq<U>,
{
    type Other = U;

    fn dir(&self) -> MapDir {
        self.dir
    }

    fn alt_dest(&self) -> Option<Coord> {
        self.alt_dest
    }

    fn change_state(&mut self, new_state: DoorState, tool: Option<T>) -> Result<(), DoorState>
    where
        T: PartialEq<Self::Other>,
    {
        if self.state == new_state {
            return Ok(());
        }

        match (self.keyhole.clone(), tool) {
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
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
    MagicallySealed,
    PermaLocked,
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
pub struct Door<U: Debug> {
    dir: MapDir,
    state: DoorState,
    alt_dest: Option<Coord>,
    keyhole: Option<U>,
}

impl<U: Debug> ObstacleState<DoorState> for Door<U> {
    fn state(&self) -> DoorState {
        self.state.clone()
    }

    fn is_blocked(&self) -> bool {
        match self.state {
            Open => false,
            _ => true,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DoorList<U: Eq + Clone + Debug>(Vec<Door<U>>);

impl<U: Eq + Clone + Debug> Deref for DoorList<U> {
    type Target = Vec<Door<U>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<U: Eq + Clone + Debug> DerefMut for DoorList<U> {
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
        let mut d: Door<u128> = Door {
            dir: North,
            state: DoorState::Locked,
            alt_dest: None,
            keyhole: Some(8_u128),
        };

        let n: u128 = 8;
        let res = d.change_state(DoorState::Open, Some(n));
        assert_eq!(res, Ok(()));
        assert_eq!(d.state(), DoorState::Open);
        let res = d.change_state(Locked, Some(72));
        assert_eq!(
            res,
            Err(Locked)
        );
        assert_eq!(d.state(), DoorState::Open);
        let res = d.change_state(Locked, None);
        assert_eq!(
            res,
            Err(Locked)
        );
        assert_eq!(d.state(), DoorState::Open);
    }
}
