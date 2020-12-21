use crate::item::ItemList;
use crate::map::Coord;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use uuid::Uuid as CrateUuid;

#[derive(Debug, Default)]
pub struct Player {
    uuid: u128,
    name: String,
    description: String,
    loc: Coord,
    items: ItemList,
}

#[derive(Debug, Default)]
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
}

pub trait Uuid {
    fn uuid(&self) -> u128;
}

impl Uuid for Player {
    fn uuid(&self) -> u128 {
        self.uuid()
    }
}

impl Uuid for &Player {
    fn uuid(&self) -> u128 {
        self.uuid
    }
}

impl Uuid for u128 {
    fn uuid(&self) -> u128 {
        *self
    }
}

#[derive(Default)]
pub struct PlayerList(pub HashMap<u128, Player>);
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
            uuid: CrateUuid::new_v4().as_u128(),
            description: "".to_owned(),
            name: name.to_owned(),
            loc: Coord(0, 0),
            items: ItemList::new(),
        }
    }

    pub fn set_description(&mut self, d: &str) {
        self.description = d.to_owned();
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

    pub fn items(&self) -> &ItemList {
        &self.items
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn get_itemlist(&mut self) -> ItemList {
        std::mem::take(&mut self.items)
    }

    pub fn replace_itemlist(&mut self, i: ItemList) {
        self.items = i;
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
