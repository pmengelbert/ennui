use std::io::{Read, Write};
use std::net::TcpStream;

use serde::{Deserialize, Serialize};

use meter::MeterKind;

use crate::error::EnnuiError;
use crate::item::handle::Handle;
use crate::item::list::{Holder, ItemList, ItemListTrout, ListTrait};
use crate::item::{Attribute, Describe, Description, Item, Quality};
use crate::map::coord::Coord;
use crate::map::Locate;

use crate::fight::FightMod;
use crate::fight::FightMod::Leave;
use std::error::Error;
use std::net::Shutdown::Both;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use rand::Rng;

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
    stats: Vec<MeterKind>,
    status: Vec<PlayerStatus>,
    #[serde(skip_serializing, skip_deserializing)]
    fight_sender: Option<Arc<Mutex<Sender<FightMod>>>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub enum PlayerStatus {
    Fighting,
    Dead,
    Asleep,
    Sitting,
}

#[cfg(test)]
mod test_playerstatus {
    use super::*;

    #[test]
    fn test_player_status() {
        eeprintln!("{:#?}", PlayerStatus::Asleep as u64);
    }
}

impl Attribute<PlayerStatus> for Player {
    fn attr(&self) -> Vec<PlayerStatus> {
        self.status.clone()
    }

    fn set_attr(&mut self, q: PlayerStatus) {
        self.status.push(q);
    }

    fn unset_attr(&mut self, q: PlayerStatus) {
        let pos = self.status.iter().position(|u| *u == q);
        if let Some(pos) = pos {
            self.status.remove(pos);
        }
    }
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

impl Locate for Arc<Mutex<Player>> {
    fn loc(&self) -> Coord {
        self.lock().unwrap().loc
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
    fn name(&self) -> String {
        self.info.name()
    }

    fn display(&self) -> String {
        self.info.display()
    }

    fn description(&self) -> String {
        self.info.description()
    }

    fn handle(&self) -> Handle {
        self.info.handle()
    }
}

impl Describe for Arc<Mutex<Player>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }

    fn display(&self) -> String {
        self.lock().unwrap().display()
    }

    fn description(&self) -> String {
        self.lock().unwrap().description()
    }

    fn handle(&self) -> Handle {
        self.lock().unwrap().handle()
    }
}

impl Attribute<Quality> for Arc<Mutex<Player>> {
    fn attr(&self) -> Vec<Quality> {
        self.lock().unwrap().attr()
    }

    fn set_attr(&mut self, q: Quality) {
        self.lock().unwrap().set_attr(q)
    }

    fn unset_attr(&mut self, q: Quality) {
        self.lock().unwrap().unset_attr(q)
    }
}

impl Attribute<PlayerStatus> for Arc<Mutex<Player>> {
    fn attr(&self) -> Vec<PlayerStatus> {
        self.lock().unwrap().status.clone()
    }

    fn set_attr(&mut self, q: PlayerStatus) {
        self.lock().unwrap().set_attr(q);
    }

    fn unset_attr(&mut self, q: PlayerStatus) {
        self.lock().unwrap().unset_attr(q);
    }
}

impl Attribute<Quality> for Player {
    fn attr(&self) -> Vec<Quality> {
        self.info.attributes.clone()
    }

    fn set_attr(&mut self, q: Quality) {
        self.info.set_attr(q);
    }

    fn unset_attr(&mut self, q: Quality) {
        self.info.unset_attr(q);
    }
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
            uuid: 10,
            info: Description {
                description: String::new(),
                name: String::new(),
                handle: Handle(vec![]),
                display: String::new(),
                attributes: vec![],
            },
            loc: Coord(0, 0),
            items: ItemList::new(),
            clothing: ItemList::new(),
            stream: None,
            fight_sender: None,
            status: vec![],
            stats,
        }
    }

    pub fn new_with_stream(stream: TcpStream) -> Self {
        let mut p = Self::new();
        p.assign_stream(stream);
        p
    }

    pub fn set_name(&mut self, name: &str) {
        self.info.handle.push(name.to_owned());
        self.info.name = name.to_owned();
    }

    pub fn hurt(&mut self, amt: usize) {
        use meter::MeterKind::*;
        let current = self.hp();
        (*self.stats.iter_mut().find(|s| matches!(s, Hit(_))).unwrap()).set(current - amt as i64);
    }

    pub fn hp(&self) -> i64 {
        use meter::MeterKind::*;
        self.stats
            .iter()
            .find(|s| matches!(s, Hit(_)))
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

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn drop_stream(&mut self) {
        self.stream = None
    }

    pub fn assign_fight_sender(&mut self, sender: Sender<FightMod>) {
        self.fight_sender = Some(Arc::new(Mutex::new(sender)));
    }

    pub fn leave_fight(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(sender) = self.fight_sender.take() {
            let sender = sender.lock().unwrap();
            return Ok(sender.send(Leave(self.uuid))?);
        }
        Ok(())
    }

    fn assign_stream(&mut self, stream: TcpStream) {
        self.stream = Some(stream);
    }
}
