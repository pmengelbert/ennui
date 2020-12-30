use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::item::Describe;
use crate::player::{Player, Uuid};
use crate::text::message::Messenger;

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

    fn others(&self) -> Option<Vec<u128>> {
        let v: Vec<u128> = self.iter().cloned().collect();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
}

impl Messenger for PlayerIdList {
    fn id(&self) -> Option<u128> {
        None
    }

    fn others(&self) -> Option<Vec<u128>> {
        Some(self.iter().cloned().collect())
    }
}

impl PlayerIdList {
    pub fn get_player_by_name<'a>(&self, pl: &'a PlayerList, name: &str) -> Option<&'a Player> {
        let u = self.id_of_name(pl, name)?;
        pl.get(&u)
    }

    pub fn get_player_mut_by_name<'a>(
        &self,
        pl: &'a mut PlayerList,
        name: &str,
    ) -> Option<&'a mut Player> {
        let u = self.id_of_name(pl, name)?;
        pl.get_mut(&u)
    }

    fn id_of_name(&self, g: &PlayerList, name: &str) -> Option<u128> {
        Some(
            *self
                .iter()
                .find(|p| g.get(p).unwrap_or(&Player::new("")).name() == name)?,
        )
    }

    pub fn except(&self, id: u128) -> PlayerIdList {
        let mut cl = self.clone();
        cl.remove(&id);
        cl
    }
}

#[derive(Default)]
#[repr(transparent)]
pub struct PlayerList(pub HashMap<u128, Player>);

impl Deref for PlayerList {
    type Target = HashMap<u128, Player>;

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

    fn others(&self) -> Option<Vec<u128>> {
        let v: Vec<_> = self.keys().cloned().collect();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
}

impl Uuid for &PlayerIdList {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Option<Vec<u128>> {
        let v: Vec<u128> = self.iter().cloned().collect();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
}

impl Uuid for &PlayerList {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Option<Vec<u128>> {
        let v: Vec<_> = self.keys().cloned().collect();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
}
