use std::io::{Read, Write};
use std::net::TcpStream;

use serde::{Deserialize, Serialize};

use meter::MeterKind;

use crate::attribute::{Attribute, Quality};
use crate::describe::{Describe, Description};
use crate::error::EnnuiError;
use crate::hook::{Grabber, Hook};
use crate::item::Item;
use crate::list::{List, ListTrait};
use crate::location::{Coord, Locate};
use crate::soul::SoulKind;

use crate::fight::FightMod;
use crate::fight::FightMod::Leave;

#[cfg(not(target_arch = "wasm32"))]
use rand::{thread_rng, Rng};
use std::error::Error;
use std::net::Shutdown::Both;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub mod list;
mod meter;
pub mod npc;
mod player_test;
use npc::YamlPlayer;

#[derive(Debug)]
pub enum PlayerType {
    Human(Player),
    Npc(npc::Npc),
    Dummy(Player),
}

impl Default for PlayerType {
    fn default() -> Self {
        PlayerType::Dummy(Player::default())
    }
}

impl From<YamlPlayer> for PlayerType {
    fn from(other: YamlPlayer) -> Self {
        let mut p = Player::new();
        let YamlPlayer {
            info,
            ai_type,
            loc,
            soul,
        } = other;
        p.info = info;
        p.loc = loc;
        if let Some(t) = ai_type {
            Self::Npc(npc::Npc::new(p, t, soul))
        } else {
            Self::Human(p)
        }
    }
}

impl PlayerType {
    fn safe_unwrap(&self) -> &Player {
        use PlayerType::*;
        match self {
            Human(ref h) => h,
            Npc(npc) => npc.player(),
            Dummy(ref p) => p,
        }
    }

    fn safe_unwrap_mut(&mut self) -> &mut Player {
        use PlayerType::*;
        match self {
            Human(ref mut h) => h,
            Npc(npc) => npc.player_mut(),
            Dummy(ref mut p) => p,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Player {
    uuid: u128,
    info: Description,
    loc: Coord,
    #[serde(skip_serializing, skip_deserializing)]
    items: List<Item, Quality>,
    #[serde(skip_deserializing, skip_serializing)]
    souls: List<SoulKind, Quality>,
    #[serde(skip_serializing, skip_deserializing)]
    clothing: List<Item, Quality>,
    #[serde(skip_serializing, skip_deserializing)]
    pub stream: Option<TcpStream>,
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

impl Attribute<PlayerStatus> for PlayerType {
    fn attr(&self) -> Vec<PlayerStatus> {
        self.safe_unwrap().status.clone()
    }

    fn set_attr(&mut self, q: PlayerStatus) {
        self.safe_unwrap_mut().status.push(q);
    }

    fn unset_attr(&mut self, q: PlayerStatus) {
        let pos = self.safe_unwrap().status.iter().position(|u| *u == q);
        if let Some(pos) = pos {
            self.safe_unwrap_mut().status.remove(pos);
        }
    }
}

pub trait Uuid {
    fn uuid(&self) -> u128;
    fn others(&self) -> Vec<u128> {
        vec![]
    }
}

impl Read for PlayerType {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.safe_unwrap_mut()
            .stream
            .as_ref()
            .ok_or(std::io::ErrorKind::BrokenPipe)?
            .read(buf)
    }
}

impl Drop for PlayerType {
    fn drop(&mut self) {
        match &self.safe_unwrap_mut().stream {
            Some(s) => {
                s.shutdown(Both).unwrap_or_default();
            }
            None => (),
        }
    }
}

impl ListTrait for PlayerType {
    type Item = Item;

    fn get_item(&self, handle: Grabber) -> Option<&Self::Item> {
        self.safe_unwrap().items.get_item(handle)
    }

    fn get_item_mut(&mut self, handle: Grabber) -> Option<&mut Self::Item> {
        self.safe_unwrap_mut().items.get_item_mut(handle)
    }

    fn get_item_owned(&mut self, handle: Grabber) -> Result<Self::Item, EnnuiError> {
        self.safe_unwrap_mut().items.get_item_owned(handle)
    }

    fn insert_item(&mut self, item: Self::Item) -> Result<(), Self::Item> {
        self.safe_unwrap_mut().items.insert_item(item)
    }

    fn display_items(&self) -> String {
        self.safe_unwrap().items.display_items()
    }

    fn list(&self) -> Vec<&Self::Item> {
        self.safe_unwrap().items.list()
    }
}

impl Locate for PlayerType {
    fn loc(&self) -> Coord {
        self.safe_unwrap().loc
    }
}

impl Locate for Arc<Mutex<PlayerType>> {
    fn loc(&self) -> Coord {
        self.lock().unwrap().loc()
    }
}

impl Uuid for PlayerType {
    fn uuid(&self) -> u128 {
        self.safe_unwrap().uuid
    }
}

impl Describe for PlayerType {
    fn name(&self) -> String {
        self.safe_unwrap().info.name()
    }

    fn display(&self) -> String {
        self.safe_unwrap().info.display()
    }

    fn description(&self) -> String {
        self.safe_unwrap().info.description()
    }

    fn handle(&self) -> Hook {
        self.safe_unwrap().info.handle()
    }
}

impl Describe for Arc<Mutex<PlayerType>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }

    fn display(&self) -> String {
        self.lock().unwrap().display()
    }

    fn description(&self) -> String {
        self.lock().unwrap().description()
    }

    fn handle(&self) -> Hook {
        self.lock().unwrap().handle()
    }
}

impl Attribute<PlayerStatus> for Arc<Mutex<PlayerType>> {
    fn attr(&self) -> Vec<PlayerStatus> {
        self.lock().unwrap().attr()
    }

    fn set_attr(&mut self, q: PlayerStatus) {
        self.lock().unwrap().set_attr(q);
    }

    fn unset_attr(&mut self, q: PlayerStatus) {
        self.lock().unwrap().unset_attr(q);
    }
}

impl Write for PlayerType {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.safe_unwrap_mut().stream {
            Some(ref mut s) => s.write(buf),
            None => Ok(0),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.safe_unwrap_mut().stream {
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

        let uuid = new_player_id();

        let mut souls: List<SoulKind, Quality> = List::new();
        souls.insert_item(SoulKind::Crafting);
        souls.insert_item(SoulKind::Exploration);

        Self {
            uuid,
            info: Description {
                description: String::new(),
                name: String::new(),
                handle: Hook(vec![]),
                display: String::new(),
            },
            loc: Coord(0, 0),
            items: List::new(),
            souls,
            clothing: List::new(),
            stream: None,
            fight_sender: None,
            status: vec![],
            stats,
        }
    }

    fn assign_stream(&mut self, stream: TcpStream) {
        self.stream = Some(stream);
    }
}

impl PlayerType {
    pub fn new_with_stream(stream: TcpStream) -> Self {
        let mut p = Player::new();
        p.assign_stream(stream);
        Self::Human(p)
    }

    pub fn set_name(&mut self, name: &str) {
        self.safe_unwrap_mut().info.handle.push(name.to_lowercase());
        self.safe_unwrap_mut().info.name = name.to_owned();
        self.safe_unwrap_mut().info.display = name.to_owned();
    }

    pub fn hurt(&mut self, amt: usize) {
        use meter::MeterKind::*;
        let current = self.hp();
        (*self
            .safe_unwrap_mut()
            .stats
            .iter_mut()
            .find(|s| matches!(s, Hit(_)))
            .unwrap())
        .set(current - amt as i64);
    }

    pub fn hp(&self) -> i64 {
        use meter::MeterKind::*;
        self.stats()
            .iter()
            .find(|s| matches!(s, Hit(_)))
            .unwrap()
            .current()
    }

    pub fn set_loc(&mut self, new_loc: Coord) {
        self.safe_unwrap_mut().loc = new_loc;
    }

    pub fn clothing(&self) -> &List<Item, Quality> {
        &self.safe_unwrap().clothing
    }

    pub fn clothing_mut(&mut self) -> &mut List<Item, Quality> {
        &mut self.safe_unwrap_mut().clothing
    }

    pub fn souls(&self) -> &List<SoulKind, Quality> {
        &self.safe_unwrap().souls
    }

    pub fn souls_mut(&mut self) -> &mut List<SoulKind, Quality> {
        &mut self.safe_unwrap_mut().souls
    }

    pub fn all_items_mut(&mut self) -> (&mut List<Item, Quality>, &mut List<Item, Quality>) {
        let unwrapped = self.safe_unwrap_mut();
        (&mut unwrapped.items, &mut unwrapped.clothing)
    }

    pub fn stats(&self) -> &[MeterKind] {
        &self.safe_unwrap().stats
    }

    pub fn clone_stream(&self) -> Option<TcpStream> {
        self.safe_unwrap()
            .stream
            .as_ref()
            .map(|s| s.try_clone().unwrap())
    }

    pub fn is_connected(&self) -> ConnectionStatus {
        match self {
            PlayerType::Npc(_) => ConnectionStatus::Npc,
            PlayerType::Human(p) => {
                if p.stream.is_some() {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::Disconnected
                }
            }
            PlayerType::Dummy(_) => ConnectionStatus::None,
        }
    }

    pub fn drop_stream(&mut self) {
        self.safe_unwrap_mut().stream = None
    }

    pub fn assign_fight_sender(&mut self, sender: Sender<FightMod>) {
        self.safe_unwrap_mut().fight_sender = Some(Arc::new(Mutex::new(sender)));
    }

    pub fn leave_fight(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(sender) = self.safe_unwrap_mut().fight_sender.take() {
            let sender = sender.lock().unwrap();
            return Ok(sender.send(Leave(self.uuid()))?);
        }
        Ok(())
    }
}

pub enum ConnectionStatus {
    Npc,
    Connected,
    Disconnected,
    None,
}

#[cfg(not(target_arch = "wasm32"))]
fn new_player_id() -> u128 {
    thread_rng().gen_range(0, u128::MAX)
}

#[cfg(target_arch = "wasm32")]
pub const PLAYER_ID: u128 = 10;

#[cfg(target_arch = "wasm32")]
fn new_player_id() -> u128 {
    PLAYER_ID
}
