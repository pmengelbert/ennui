use crate::item::ItemList;
use crate::map::Coord;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use uuid::Uuid as CrateUuid;
use std::net::TcpStream;
use std::io::Write;

#[derive(Debug, Default)]
pub struct Player {
    uuid: u128,
    name: String,
    description: String,
    loc: Coord,
    items: ItemList,
    clothing: ItemList,
    stream: Option<TcpStream>,
}

impl Write for Player {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.stream {
             Some(ref mut s) => s.write(buf),
             None => Ok(0)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.stream {
            Some(ref mut s) => s.flush(),
            None => Ok(())
        }
    }
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
            clothing: ItemList::new(),
            stream: None,
        }
    }

    pub fn new_with_stream(name: &str, stream: TcpStream) -> Self {
        let mut p = Self::new(name);
        p.assign_stream(stream);
        p
    }

    pub fn assign_stream(&mut self, stream: TcpStream) {
        self.stream = Some(stream);
    }

    pub fn broadcast<T>(&self, pl: &mut PlayerList, buf: T) -> std::io::Result<usize>
    where
        T: AsRef<[u8]>
    {
        let mut result = 0_usize;
        for (_, p) in &mut **pl {
            let mut retstr = b"\n\n".to_vec();
            retstr.extend_from_slice(format!("{} chats '", self.name).as_bytes());
            retstr.extend_from_slice(buf.as_ref());
            retstr.extend_from_slice(&b"'\n\n > "[..]);
            result = p.write(&retstr)?;
            println!("{}", result);
            p.flush()?;
        }
        Ok(result)
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

    pub fn get_clothinglist(&mut self) -> ItemList {
        std::mem::take(&mut self.clothing)
    }

    pub fn replace_itemlist(&mut self, i: ItemList) {
        self.items = i;
    }

    pub fn replace_clothinglist(&mut self, i: ItemList) {
        self.clothing = i;
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
