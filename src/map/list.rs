use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::map::coord::Coord;
use crate::map::direction::MapDir;
use crate::map::Room;
use crate::player::list::PlayerIdList;
use crate::player::Uuid;

#[repr(transparent)]
#[derive(Default, Deserialize, Serialize, Debug)]
pub struct RoomList(HashMap<Coord, Room>);

impl Uuid for RoomList {
    fn uuid(&self) -> u128 {
        0
    }
}

pub trait RoomListTrait {
    fn player_ids(&self, loc: Coord) -> PlayerIdList;
    fn exits(&self, loc: Coord) -> Vec<MapDir>;
}

impl RoomListTrait for RoomList {
    fn player_ids(&self, loc: Coord) -> PlayerIdList {
        match self.get(&loc) {
            Some(r) => r.players.clone(),
            None => PlayerIdList::default(),
        }
    }

    fn exits(&self, loc: Coord) -> Vec<MapDir> {
        MapDir::all()
            .iter()
            .filter_map(|d| {
                if self.contains_key(&loc.add(*d)?) {
                    Some(*d)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Deref for RoomList {
    type Target = HashMap<Coord, Room>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RoomList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
