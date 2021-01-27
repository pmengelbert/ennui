use std::collections::HashMap;

use crate::map::coord::Coord;
use crate::map::direction::MapDir;
use crate::map::Room;
use crate::player::list::PlayerIdList;
use crate::player::Uuid;

pub type RoomList = HashMap<Coord, Room>;

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
