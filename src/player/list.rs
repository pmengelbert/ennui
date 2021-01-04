use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::player::{Player, Uuid};
use crate::text::message::Messenger;

use crate::item::Describe;
use crate::map::coord::Coord;
use crate::text::Color::Yellow;
use std::sync::{Arc, Mutex};

#[repr(transparent)]
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct PlayerIdList(pub HashSet<u128>);

impl Deref for PlayerIdList {
    type Target = HashSet<u128>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerIdList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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

impl PlayerIdList {
    pub fn except(&self, id: u128) -> PlayerIdList {
        let mut cl = self.clone();
        cl.remove(&id);
        cl
    }
}

#[derive(Default)]
#[repr(transparent)]
pub struct PlayerList(pub HashMap<u128, Arc<Mutex<Player>>>);

impl Deref for PlayerList {
    type Target = HashMap<u128, Arc<Mutex<Player>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PlayerList {
    pub fn new() -> Self {
        PlayerList(HashMap::new())
    }
}

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

impl PlayerList {
    pub fn to_id_list(&self) -> PlayerIdList {
        let mut pil = PlayerIdList::default();
        for id in self.keys() {
            pil.insert(*id);
        }
        pil
    }

    pub fn display(&self, loc: Coord) -> Vec<String> {
        self.values()
            .filter(|p| p.lock().unwrap().loc == loc)
            .map(|p| format!("{}", Yellow(p.lock().unwrap().name().to_owned())))
            .collect()
    }
}
