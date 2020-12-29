use crate::item::handle::Handle;
use crate::item::{Describe, Description, Holder, Item, ItemList, ItemListTrait, Attribute, Quality};
use crate::map::coord::Coord;
use crate::text::message::Messenger;
use crate::Provider;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::net::TcpStream;
use std::ops::{Deref, DerefMut};
use uuid::Uuid as CrateUuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum MeterKind {
    Hit(Meter),
    Mana(Meter),
    Movement(Meter),
    Strength(Meter),
    Dexterity(Meter),
    Weight(Meter),
    Height(Meter),
}

impl<T> Provider<PlayerList> for T
where
    T: AsRef<PlayerList> + AsMut<PlayerList>,
{
    fn provide(&self) -> &PlayerList {
        self.as_ref()
    }

    fn provide_mut(&mut self) -> &mut PlayerList {
        self.as_mut()
    }
}

impl Holder for Player {
    type Kind = ItemList;

    fn items(&self) -> &Self::Kind {
        &self.items
    }

    fn items_mut(&mut self) -> &mut Self::Kind {
        &mut self.items
    }
}

impl Describe for Player {
    fn name(&self) -> &str {
        &self.info.name()
    }

    fn display(&self) -> &str {
        &self.info.display()
    }

    fn description(&self) -> &str {
        self.info.description()
    }

    fn handle(&self) -> &Handle {
        self.info.handle()
    }
}

impl Attribute<Quality> for Player {
    fn attr(&self) -> &[Quality] {
        &self.info.attributes
    }
}

impl Display for MeterKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.string().to_uppercase(),
            self.safe_unwrap()
        )
    }
}

impl AsRef<ItemList> for Player {
    fn as_ref(&self) -> &ItemList {
        self.items()
    }
}

impl AsMut<ItemList> for Player {
    fn as_mut(&mut self) -> &mut ItemList {
        self.items_mut()
    }
}

impl AsRef<Coord> for Player {
    fn as_ref(&self) -> &Coord {
        &self.loc
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Meter(pub i64, pub i64);

impl Display for Meter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} / {}]", self.0, self.1)
    }
}

impl Meter {
    pub fn current(&self) -> i64 {
        self.0
    }

    pub fn max(&self) -> i64 {
        self.1
    }

    pub fn set(&mut self, val: i64) {
        self.0 = val;
    }
}

impl MeterKind {
    pub fn current(&self) -> i64 {
        self.safe_unwrap().0
    }

    pub fn max(&self) -> i64 {
        self.safe_unwrap().1
    }

    pub fn set(&mut self, val: i64) {
        *self.safe_unwrap_mut().0 = val
    }

    fn safe_unwrap(&self) -> &Meter {
        use MeterKind::*;
        match self {
            Hit(m) | Mana(m) | Movement(m) | Strength(m) | Dexterity(m) | Weight(m) | Height(m) => {
                m
            }
        }
    }

    fn string(&self) -> &'static str {
        match self {
            MeterKind::Hit(_) => "hit",
            MeterKind::Mana(_) => "mana",
            MeterKind::Movement(_) => "movement",
            MeterKind::Strength(_) => "strength",
            MeterKind::Dexterity(_) => "dexterity",
            MeterKind::Weight(_) => "weight",
            MeterKind::Height(_) => "height",
        }
    }

    fn safe_unwrap_mut(&mut self) -> (&mut i64, &mut i64) {
        use MeterKind::*;
        match self {
            Hit(m) | Mana(m) | Movement(m) | Strength(m) | Dexterity(m) | Weight(m) | Height(m) => {
                (&mut m.0, &mut m.1)
            }
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Player {
    uuid: u128,
    info: Description,
    loc: Coord,
    #[serde(skip_serializing, skip_deserializing)]
    items: ItemList,
    #[serde(skip_serializing, skip_deserializing)]
    clothing: ItemList,
    #[serde(skip_serializing, skip_deserializing)]
    stream: Option<TcpStream>,
    stats: Vec<MeterKind>,
}

impl Write for Player {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.stream {
            Some(ref mut s) => s.write(buf),
            None => Ok(0),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.stream {
            Some(ref mut s) => s.flush(),
            None => Ok(()),
        }
    }
}

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
}

pub trait Uuid {
    fn uuid(&self) -> u128;
    fn others(&self) -> Option<Vec<u128>> {
        None
    }
}

impl Uuid for u128 {
    fn uuid(&self) -> u128 {
        *self
    }
}

impl Uuid for Player {
    fn uuid(&self) -> u128 {
        self.uuid
    }
}

impl Uuid for &Player {
    fn uuid(&self) -> u128 {
        self.uuid
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

impl Player {
    pub fn new(name: &str) -> Self {
        use MeterKind::*;
        let stats = vec![
            Hit(Meter(100, 100)),
            Movement(Meter(100, 100)),
            Mana(Meter(100, 100)),
        ];

        Self {
            uuid: CrateUuid::new_v4().as_u128(),
            info: Description {
                description: "".to_owned(),
                name: name.to_owned(),
                handle: Handle(vec![name.to_owned()]),
                display: "".to_owned(),
                attributes: vec![],
            },
            loc: Coord(0, 0),
            items: ItemList::new(),
            clothing: ItemList::new(),
            stream: None,
            stats,
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
        T: AsRef<[u8]>,
    {
        let mut result = 0_usize;
        for (_, p) in &mut **pl {
            let mut ret = b"\n\n".to_vec();
            ret.extend_from_slice(format!("{} chats '", self.name()).as_bytes());
            ret.extend_from_slice(buf.as_ref());
            ret.extend_from_slice(b"'\n\n > ".as_ref());
            result = p.write(&ret)?;
            p.flush()?;
        }
        Ok(result)
    }

    pub fn hurt(&mut self, amt: usize) {
        use MeterKind::*;
        let current = self.hp();
        (*self
            .stats
            .iter_mut()
            .find(|s| if let Hit(_) = s { true } else { false })
            .unwrap())
        .set(current - amt as i64);
    }

    pub fn hp(&self) -> i64 {
        use MeterKind::*;
        self.stats
            .iter()
            .find(|s| if let Hit(_) = s { true } else { false })
            .unwrap()
            .current()
    }

    pub fn uuid(&self) -> u128 {
        self.uuid
    }

    pub fn set_loc(&mut self, new_loc: Coord) {
        self.loc = new_loc;
    }

    pub fn loc_mut(&mut self) -> &mut Coord {
        &mut self.loc
    }

    pub fn clothing(&self) -> &ItemList {
        &self.clothing
    }

    pub fn clothing_mut(&mut self) -> &mut ItemList {
        &mut self.clothing
    }

    pub fn all_items(&self) -> (&ItemList, &ItemList) {
        (&self.items, &self.clothing)
    }

    pub fn all_items_mut(&mut self) -> (&mut ItemList, &mut ItemList) {
        (&mut self.items, &mut self.clothing)
    }

    pub fn stats(&self) -> &[MeterKind] {
        &self.stats
    }
}

#[cfg(test)]
mod player_test {
    use super::*;
    use MeterKind::*;

    #[test]
    fn player_test_uuid() {
        assert_ne!(Player::new("").uuid(), 0);
    }

    #[test]
    fn test_meter_display() {
        let x = Meter(100, 100);
        assert_eq!(format!("{}", x), "[100 / 100]");
        let y = Hit(x);
        assert_eq!(format!("{}", y), "HIT: [100 / 100]");
    }
}

impl ItemListTrait for Player {
    type Kind = ItemList;

    fn get(&self, handle: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.handle() == handle)
    }

    fn get_mut(&mut self, handle: &str) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.handle() == handle)
    }

    fn get_owned(&mut self, handle: &str) -> Option<Item> {
        let pos = self.items().iter().position(|i| i.handle() == handle)?;
        Some(self.items.remove(pos))
    }

    fn insert(&mut self, item: Item) -> Result<(), Item> {
        self.items.push(item);
        Ok(())
    }

    fn list(&self) -> &Self::Kind {
        &self.items
    }
}
