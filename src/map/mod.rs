pub mod coord;
pub mod door;

use crate::game::MapDir;
use crate::item::{
    BasicItemKind, GenericItemList, Holder, Item, ItemList, ItemListTrait, ItemTrait,
};
use crate::player::{Player, PlayerIdList, PlayerList, Uuid};
use crate::text::Color::*;
use crate::text::Wrap;
use crate::{PassFail, Provider};

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use crate::item::handle::Handle;
use crate::map::coord::Coord;
use crate::map::door::{Door, DoorState, Obstacle, ObstacleState, DoorList};
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, Cow};
use std::mem::take;
use std::path::Display;
use std::sync::Arc;

type StateResult<T> = Result<(), T>;

pub trait Space: Locate + ItemListTrait {
    fn players(&self) -> &PlayerIdList;
    fn doors(&mut self) -> &mut DoorList;
    fn players_except(&self, u: u128) -> Vec<u128> {
        let u = u.uuid();
        let mut l = Vec::new();
        for &id in self.players().iter() {
            if id == u {
                continue;
            }

            l.push(id);
        }
        l
    }

}

impl<T> Provider<RoomList> for T
where
    T: AsRef<RoomList> + AsMut<RoomList>,
{
    fn provide(&self) -> &RoomList {
        self.as_ref()
    }

    fn provide_mut(&mut self) -> &mut RoomList {
        self.as_mut()
    }
}

impl Uuid for Room {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Option<Vec<u128>> {
        let mut v = vec![];
        for id in self.players.iter() {
            if *id == self.uuid() {
                continue;
            }
            v.push(*id)
        }

        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
}

impl Uuid for &Room {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Option<Vec<u128>> {
        let mut v = vec![];
        for id in self.players.iter() {
            if *id == self.uuid() {
                continue;
            }
            v.push(*id)
        }

        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
}

impl Uuid for RoomList {
    fn uuid(&self) -> u128 {
        0
    }
}

pub trait Locate {
    fn loc(&self) -> Coord;
    fn room<'a, T>(&self, rooms: &'a T) -> Option<&'a Room>
    where
        T: Provider<RoomList>,
    {
        let rooms: &RoomList = rooms.provide();
        rooms.get(&self.loc())
    }

    fn player_ids<T>(&self, rooms: &T) -> Option<PlayerIdList>
    where
        T: Provider<RoomList>,
    {
        Some(self.room(rooms)?.players().clone())
    }

    fn player_by_name<'a, T, S>(&self, provider: &'a T, other: S) -> Option<&'a Player>
    where
        T: Provider<RoomList> + Provider<PlayerList>,
        S: AsRef<str>,
    {
        let pl: &PlayerList = provider.provide();
        pl.values()
            .find(|p| p.name() == other.as_ref() && p.loc() == self.loc())
    }

    fn player_by_name_mut<'a, T, S>(&self, provider: &'a mut T, other: S) -> Option<&'a mut Player>
    where
        T: Provider<RoomList> + Provider<PlayerList>,
        S: AsRef<str>,
    {
        let pl: &mut PlayerList = provider.provide_mut();
        pl.values_mut()
            .find(|p| p.name() == other.as_ref() && p.loc() == self.loc())
    }

    fn move_player<T, U>(&self, provider: &mut T, u: U, dir: MapDir) -> StateResult<DoorState>
    where
        T: Provider<RoomList> + Provider<PlayerList>,
        U: Uuid,
    {
        let u = u.uuid();
        let next_coord = self.loc().add(dir);

        {
            let rooms: &mut RoomList = provider.provide_mut();
            {
                let src_room = rooms.get_mut(&self.loc())?;
                if let Some(door) = src_room.doors.get(&dir) {
                    match door.state() {
                        DoorState::None | DoorState::Open => (),
                        s => return Err(s),
                    }
                }
                src_room.players_mut().remove(&u);
            }
            rooms.get_mut(&next_coord?)?.players_mut().insert(u);
        }

        {
            let players: &mut PlayerList = provider.provide_mut();
            players.get_mut(&u)?.set_loc(next_coord?);
        }

        Ok(())
    }

    fn exits<T>(&self, provider: &T) -> Vec<MapDir>
    where
        T: Provider<RoomList>,
    {
        let rooms = provider.provide();
        MapDir::all()
            .iter()
            .filter_map(|d| {
                if rooms.contains_key(&self.loc().add(*d)?) {
                    Some(*d)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[repr(transparent)]
#[derive(Default, Deserialize, Serialize, Debug)]
pub struct RoomList(HashMap<Coord, Room>);
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

impl AsRef<RoomList> for RoomList {
    fn as_ref(&self) -> &RoomList {
        self
    }
}

impl AsMut<RoomList> for RoomList {
    fn as_mut(&mut self) -> &mut RoomList {
        self
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Room {
    name: String,
    loc: Coord,
    description: String,
    players: PlayerIdList,
    #[serde(skip_serializing, skip_deserializing)]
    items: ItemList,
    inner_items: Option<GenericItemList>,
    #[serde(default)]
    doors: DoorList,
    #[serde(skip_serializing, skip_deserializing)]
    handle: Handle,
}

impl AsMut<ItemList> for Room {
    fn as_mut(&mut self) -> &mut ItemList {
        self.items_mut()
    }
}

impl AsRef<ItemList> for Room {
    fn as_ref(&self) -> &ItemList {
        self.items()
    }
}

impl AsRef<Coord> for Room {
    fn as_ref(&self) -> &Coord {
        &self.loc
    }
}

impl Space for Room {
    fn players(&self) -> &PlayerIdList {
        &self.players
    }

    fn doors(&mut self) -> &mut DoorList {
        &mut self.doors
    }
}

impl Room {
    pub fn new(name: &str, description: Option<&str>, loc: Coord) -> Self {
        let name = name.to_owned();
        let description = description.unwrap_or("").to_owned();
        Self {
            name,
            description,
            loc: loc,
            players: PlayerIdList(HashSet::new()),
            items: ItemList::new(),
            inner_items: None,
            doors: DoorList(HashMap::new()),
            handle: Handle::default(),
        }
    }

    pub fn init(&mut self) {
        let mut inner = self.inner_items.take().unwrap_or_default();
        self.items = inner.into();
    }

    pub fn display(&self, p: u128, global_players: &PlayerList, rooms: &RoomList) -> String {
        let Room {
            name,
            description,
            players,
            items,
            ..
        } = self;

        let player_list = players
            .iter()
            .filter_map(|uuid| match global_players.get(uuid) {
                Some(player) if player.uuid() != p && player.uuid() != 0 => Some(player.name()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let player_list = Yellow(match player_list.len() {
            0 => "".to_owned(),
            1 => format!("\n{}", player_list[0]),
            _ => format!("\n{}", player_list.join("\n")),
        });

        let items_list = items
            .iter()
            .map(|i| i.display().to_owned())
            .collect::<Vec<String>>();

        let items_list = Green(match items_list.len() {
            0 => "".to_owned(),
            1 => format!("\n{}", items_list[0]),
            _ => format!("\n{}", items_list.join("\n")),
        });

        let exits = self.exits(rooms);

        let mut exit_str = String::from("\n[");
        for (i, dir) in exits.iter().enumerate() {
            exit_str.push_str(&format!("{}", dir));
            if i != exits.len() - 1 {
                exit_str.push_str(", ");
            }
        }
        exit_str.push(']');

        format!(
            "{}\n    {}\
            {}\
            {}\
            {}",
            Cyan(name.to_owned()),
            description,
            items_list,
            player_list,
            exit_str,
        )
    }

    pub fn players_mut(&mut self) -> &mut PlayerIdList {
        &mut self.players
    }

    pub fn add_player<P>(&mut self, p: &P) -> bool
    where
        P: Uuid,
    {
        self.players.insert(p.uuid())
    }

    pub fn get_item(&self, handle: &str) -> Option<&Item> {
        self.items().get(handle)
    }

    pub fn items(&self) -> &ItemList {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut ItemList {
        &mut self.items
    }
}

#[cfg(test)]
mod room_test {
    use super::*;
    use crate::player::Player;
}

#[cfg(test)]
mod map_test {
    use super::*;
    use crate::game::MapDir::South;
    use crate::item::BasicItem;
    use crate::item::BasicItemKind::Clothing;

    #[test]
    fn map_test() {
        assert_eq!(Coord(0, 0).north(), Coord(0, 1));
    }

    #[test]
    fn locate() {
        assert_eq!(Coord(0, 0).loc(), Coord(0, 0));
    }

    #[test]
    fn write_sample_yaml() {
        // let mut doors = HashMap::new();
        // doors.insert(South, Door {});
        // let r = Room {
        //     name: "".to_string(),
        //     loc: Default::default(),
        //     description: "".to_string(),
        //     players: Default::default(),
        //     items: Default::default(),
        //     doors,
        // };

        let mut r = Room::default();
        let mut items = GenericItemList::new();
        items.push(BasicItemKind::Weapon(BasicItem::default()));
        let mut items2 = GenericItemList::new();
        items2.push(BasicItemKind::Weapon(BasicItem::new(
            "butt",
            None,
            Handle(vec!["but".to_owned()]),
        )));
        items.push(BasicItemKind::Container(items2));
        r.inner_items = Some(items);
        std::fs::write("/tmp/sample.yaml", &serde_yaml::to_vec(&r).expect("eerr"));
    }
}

impl Holder for Room {
    type Kind = ItemList;

    fn items(&self) -> &ItemList {
        &self.items
    }

    fn items_mut(&mut self) -> &mut ItemList {
        &mut self.items
    }
}

impl ItemTrait for Room {
    fn name(&self) -> &str {
        &self.name
    }

    fn display(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn handle(&self) -> &Handle {
        &self.handle
    }

    fn is_container(&self) -> bool {
        false
    }
}

impl ItemListTrait for Room {
    type Kind = ItemList;

    fn get(&self, handle: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.handle() == handle)
    }

    fn get_mut(&mut self, handle: &str) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.handle() == handle)
    }

    fn get_owned(&mut self, handle: &str) -> Option<Item> {
        let pos = self.items.iter().position(|i| i.handle() == handle)?;
        Some(self.items.remove(pos))
    }

    fn insert(&mut self, item: Item) {
        self.items.push(item);
    }

    fn list(&self) -> &Self::Kind {
        &self.items
    }
}
