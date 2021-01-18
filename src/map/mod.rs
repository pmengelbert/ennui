use serde::{Deserialize, Serialize};

use direction::MapDir;

use crate::error::EnnuiError;
use crate::item::handle::{Grabber, Hook};
use crate::item::list::{Holder, ItemList, ItemListTrout, ListTrait};
use crate::item::{Attribute, Describe, Description, Item, Quality, YamlItemList};
use crate::map::coord::Coord;
use crate::map::door::DoorList;
use crate::player::list::PlayerIdList;
use crate::player::Uuid;
use crate::text::message::MessageFormat;
use crate::text::Color::{Cyan, Green};

pub mod coord;
pub mod direction;
pub mod door;
pub mod list;

type StateResult<T> = Result<(), T>;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Room {
    info: Description,
    loc: Coord,
    #[serde(default)]
    players: PlayerIdList,
    #[serde(skip_serializing, skip_deserializing)]
    items: ItemList,
    #[serde(default)]
    inner_items: Option<YamlItemList>,
    #[serde(default)]
    doors: DoorList,
}

pub trait Space: Locate + ListTrait {
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
    fn name(&self) -> String {
        self.info.name()
    }

    fn display(&self) -> String {
        self.info.display()
    }

    fn description(&self) -> String {
        self.info.description()
    }

    fn handle(&self) -> Hook {
        self.info.handle()
    }
}

impl Attribute<Quality> for Room {
    fn attr(&self) -> Vec<Quality> {
        self.info.attributes.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.info.set_attr(q)
    }

    fn unset_attr(&mut self, q: Quality) {
        self.info.unset_attr(q);
    }
}

impl ListTrait for Room {
    type Kind = ItemList;

    fn get_item(&self, handle: Grabber) -> Option<&Item> {
        self.items
            .iter()
            .filter(|i| i.handle() == handle.handle)
            .nth(handle.index)
    }

    fn get_item_mut(&mut self, handle: Grabber) -> Option<&mut Item> {
        self.items.get_item_mut(handle)
    }

    fn get_item_owned(&mut self, handle: Grabber) -> Result<Item, EnnuiError> {
        self.items.get_owned(handle)
    }

    fn insert_item(&mut self, item: Item) -> Result<(), Item> {
        self.items.push(item);
        Ok(())
    }

    fn list(&self) -> &Self::Kind {
        &self.items
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

    fn others(&self) -> Vec<u128> {
        let mut v = vec![];
        for id in self.players.iter() {
            if *id == self.uuid() {
                continue;
            }
            v.push(*id)
        }

        v
    }
}

pub trait Locate {
    fn loc(&self) -> Coord;
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
    pub fn init(&mut self) {
        let inner = self.inner_items.take().unwrap_or_default();
        self.items = inner.into();
    }

    pub fn doors(&self) -> &DoorList {
        &self.doors
    }

    pub fn display(&self) -> String {
        eprintln!("[{}]: room.display", "SUCCESS".color(Green));
        eprintln!("in file {} on line number {}", file!(), line!());

        let Room {
            info: Description {
                name, description, ..
            },
            items,
            ..
        } = self;

        let items_list = items.display_items();

        format!(
            "{}\n    {}\
            {}",
            name.color(Cyan),
            description,
            items_list,
        )
    }

    pub fn exit_display(exits: &[MapDir]) -> String {
        let mut exit_str = String::from("\n[");
        for (i, dir) in exits.iter().enumerate() {
            exit_str.push_str(&format!("{}", dir));
            if i != exits.len() - 1 {
                exit_str.push_str(", ");
            }
        }
        exit_str.push(']');
        exit_str
    }

    pub fn players_mut(&mut self) -> &mut PlayerIdList {
        &mut self.players
    }

    pub fn add_player<P: Uuid>(&mut self, p: &P) -> bool {
        self.players.insert(p.uuid())
    }
}
