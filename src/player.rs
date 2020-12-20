use uuid::Uuid;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use crate::map::Coord;

#[derive(Debug)]
pub struct Player {
    uuid: u128,
    name: String,
    loc: Coord,
}

pub trait UuidProvide {
    fn uuid(&self) -> u128;
}

impl UuidProvide for Player {
    fn uuid(&self) -> u128 {
        self.uuid()
    }
}

impl UuidProvide for u128 {
    fn uuid(&self) -> u128 {
        *self
    }
}

pub struct PlayerList(HashMap<u128, Player>);
pub type PlayerListRaw = HashMap<u128, Player>;

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

impl Player {
    pub fn new(name: &str) -> Self {
        Self {
            uuid: Uuid::new_v4().as_u128(),
            name: name.to_owned(),
            loc: Coord(0, 0),
        }
    }

    pub fn uuid(&self) -> u128 {
        self.uuid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn loc(&self) -> &Coord {
        &self.loc
    }
}

#[cfg(test)]
mod player_test {
    use super::*;

    #[test]
    fn player_test_uuid() {
        assert_ne!(Player::new("").uuid(), 0);
    }
}