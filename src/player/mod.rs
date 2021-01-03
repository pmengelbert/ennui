use std::io::{Read, Write};
use std::net::TcpStream;

use serde::{Deserialize, Serialize};
use uuid::Uuid as CrateUuid;

use meter::MeterKind;

use crate::error::EnnuiError;
use crate::item::handle::Handle;
use crate::item::list::{Holder, ItemList, ListTrait};
use crate::item::{Attribute, Describe, Description, Item, Quality};
use crate::map::coord::Coord;
use crate::map::Locate;

use std::net::Shutdown::Both;

pub mod list;
mod meter;
mod player_test;

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
    // drop_game: Arc<Mutex<Game>>,
    stats: Vec<MeterKind>,
}

pub trait Uuid {
    fn uuid(&self) -> u128;
    fn others(&self) -> Vec<u128> {
        vec![]
    }
}

impl Read for Player {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream
            .as_ref()
            .ok_or(std::io::ErrorKind::BrokenPipe)?
            .read(buf)
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        match &self.stream {
            Some(s) => {
                s.shutdown(Both).unwrap_or_default();
            }
            None => (),
        }
    }
}

impl ListTrait for Player {
    type Kind = ItemList;

    fn get_item(&self, handle: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.handle() == handle)
    }

    fn get_item_mut(&mut self, handle: &str) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.handle() == handle)
    }

    fn get_item_owned(&mut self, handle: &str) -> Result<Item, EnnuiError> {
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

impl Locate for Player {
    fn loc(&self) -> Coord {
        self.loc
    }
}

impl Uuid for Player {
    fn uuid(&self) -> u128 {
        self.uuid
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

impl Write for Player {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let res = match self.stream {
            Some(ref mut s) => s.write(buf),
            None => Ok(0),
        };
        res
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.stream {
            Some(ref mut s) => s.flush(),
            None => Ok(()),
        }
    }
}

impl Uuid for u128 {
    fn uuid(&self) -> u128 {
        *self
    }
}

impl Player {
    pub fn new() -> Self {
        use meter::Meter;
        use meter::MeterKind::*;
        let stats = vec![
            Hit(Meter(100, 100)),
            Movement(Meter(100, 100)),
            Mana(Meter(100, 100)),
        ];

        Self {
            uuid: CrateUuid::new_v4().as_u128(),
            info: Description {
                description: "".to_owned(),
                name: String::new(),
                handle: Handle(vec![]),
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

    pub fn new_with_stream(stream: TcpStream) -> Self {
        let mut p = Self::new();
        p.assign_stream(stream);
        p
    }

    pub fn set_name(&mut self, name: &str) {
        self.info.name = name.to_owned();
    }

    pub fn hurt(&mut self, amt: usize) {
        use meter::MeterKind::*;
        let current = self.hp();
        (*self
            .stats
            .iter_mut()
            .find(|s| if let Hit(_) = s { true } else { false })
            .unwrap())
        .set(current - amt as i64);
    }

    pub fn hp(&self) -> i64 {
        use meter::MeterKind::*;
        self.stats
            .iter()
            .find(|s| if let Hit(_) = s { true } else { false })
            .unwrap()
            .current()
    }

    pub fn set_loc(&mut self, new_loc: Coord) {
        self.loc = new_loc;
    }

    pub fn clothing(&self) -> &ItemList {
        &self.clothing
    }

    pub fn clothing_mut(&mut self) -> &mut ItemList {
        &mut self.clothing
    }

    pub fn all_items_mut(&mut self) -> (&mut ItemList, &mut ItemList) {
        (&mut self.items, &mut self.clothing)
    }

    pub fn stats(&self) -> &[MeterKind] {
        &self.stats
    }

    pub fn clone_stream(&self) -> Option<TcpStream> {
        self.stream.as_ref().map(|s| s.try_clone().unwrap())
    }

    fn assign_stream(&mut self, stream: TcpStream) {
        self.stream = Some(stream);
    }
}
