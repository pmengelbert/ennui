use std::collections::{HashMap, HashSet};

use crate::player::{PlayerType, Uuid};
use crate::text::message::{MessageFormat, Messenger};

use crate::describe::Describe;
use crate::map::{coord::Coord, Locate};
use crate::text::Color;
use crate::text::Color::Yellow;
use std::sync::{Arc, Mutex};

pub type PlayerIdList = HashSet<u128>;

impl Uuid for PlayerIdList {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Vec<u128> {
        self.iter().cloned().collect()
    }
}

impl Messenger for PlayerIdList {
    fn id(&self) -> Option<u128> {
        None
    }

    fn others(&self) -> Vec<u128> {
        self.iter().cloned().collect()
    }
}

pub trait PlayerIdListTrait {
    fn except(&self, id: u128) -> PlayerIdList;
    fn as_players(&self, players: &PlayerList) -> Vec<Arc<Mutex<PlayerType>>>;
    fn display(&self, players: &PlayerList) -> String;
}

impl PlayerIdListTrait for PlayerIdList {
    fn except(&self, id: u128) -> PlayerIdList {
        let mut cl = self.clone();
        cl.remove(&id);
        cl
    }

    fn as_players(&self, players: &PlayerList) -> Vec<Arc<Mutex<PlayerType>>> {
        players.from_ids(self)
    }

    fn display(&self, players: &PlayerList) -> String {
        use Color::*;
        players
            .from_ids(self)
            .iter()
            .map(|p| p.display().color(Yellow).custom_padded("\n", ""))
            .collect::<Vec<_>>()
            .join("")
    }
}

pub type PlayerList = HashMap<u128, Arc<Mutex<PlayerType>>>;

impl Uuid for PlayerList {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Vec<u128> {
        self.keys().cloned().collect()
    }
}

impl Uuid for &PlayerIdList {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Vec<u128> {
        self.iter().cloned().collect()
    }
}

impl Uuid for &PlayerList {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Vec<u128> {
        self.keys().cloned().collect()
    }
}

pub trait PlayerListTrait {
    fn to_id_list(&self) -> PlayerIdList;
    fn display(&self, loc: Coord) -> Vec<String>;
    fn from_ids(&self, ids: &PlayerIdList) -> Vec<Arc<Mutex<PlayerType>>>;
}

impl PlayerListTrait for PlayerList {
    fn to_id_list(&self) -> PlayerIdList {
        let mut pil = PlayerIdList::default();
        for id in self.keys() {
            pil.insert(*id);
        }
        pil
    }

    fn display(&self, loc: Coord) -> Vec<String> {
        self.values()
            .filter(|p| p.loc() == loc)
            .map(|p| p.lock().unwrap().name().color(Yellow))
            .collect()
    }

    fn from_ids(&self, ids: &PlayerIdList) -> Vec<Arc<Mutex<PlayerType>>> {
        ids.iter()
            .filter_map(|id| Some(self.get(&id)?.clone()))
            .collect()
    }
}
