use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use direction::MapDir;

use crate::item::handle::Handle;
use crate::item::list::{Holder, ItemList, ItemListTrait};
use crate::item::{Attribute, Describe, Description, Item, Quality, YamlItemList};
use crate::map::coord::Coord;
use crate::map::door::DoorList;
use crate::player::list::{PlayerIdList, PlayerList};
use crate::player::Uuid;
use crate::text::Color::*;

pub mod coord;
pub mod direction;
pub mod door;

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

impl Locate for Room {
    fn loc(&self) -> Coord {
        self.loc
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

pub trait Locate {
    fn loc(&self) -> Coord;
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
    info: Description,
    loc: Coord,
    players: PlayerIdList,
    #[serde(skip_serializing, skip_deserializing)]
    items: ItemList,
    inner_items: Option<YamlItemList>,
    #[serde(default)]
    doors: DoorList,
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
            info: Description {
                name: name.clone(),
                description,
                handle: Handle(vec![name.clone()]),
                display: "".to_owned(),
                attributes: vec![],
            },
            loc: loc,
            players: PlayerIdList(HashSet::new()),
            items: ItemList::new(),
            inner_items: None,
            doors: DoorList(HashMap::new()),
        }
    }

    pub fn init(&mut self) {
        let inner = self.inner_items.take().unwrap_or_default();
        self.items = inner.into();
    }

    pub fn doors(&self) -> &DoorList {
        &self.doors
    }

    pub fn display(&self, p: u128, global_players: &PlayerList, exits: &[MapDir]) -> String {
        let Room {
            info: Description {
                name, description, ..
            },
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
mod room_test {}

#[cfg(test)]
mod map_test {

    use crate::item::{Description, YamlItem};

    use super::*;

    #[test]
    fn map_test() {
        assert_eq!(Coord(0, 0).north(), Coord(0, 1));
    }

    #[test]
    fn write_sample_yaml() {
        let mut r = Room::default();
        let mut items = YamlItemList::new();
        items.push(YamlItem::Weapon(Description::default()));
        let mut items2 = YamlItemList::new();
        items2.push(YamlItem::Weapon(Description::new(
            "butt",
            None,
            Handle(vec!["but".to_owned()]),
        )));
        items.push(YamlItem::Container(items2));
        r.inner_items = Some(items);
        std::fs::write("/tmp/sample.yaml", &serde_yaml::to_vec(&r).expect("err"));
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

impl Describe for Room {
    fn name(&self) -> &str {
        &self.info.name()
    }

    fn display(&self) -> &str {
        &self.info.display()
    }

    fn description(&self) -> &str {
        &self.info.description()
    }

    fn handle(&self) -> &Handle {
        &self.info.handle()
    }
}

impl Attribute<Quality> for Room {
    fn attr(&self) -> &[Quality] {
        &self.info.attributes
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

    fn insert(&mut self, item: Item) -> Result<(), Item> {
        self.items.push(item);
        Ok(())
    }

    fn list(&self) -> &Self::Kind {
        &self.items
    }
}
