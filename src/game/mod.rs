use std::borrow::{BorrowMut, Cow};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io;
use std::io::Write;
use std::option::NoneError;
use std::sync::{Arc, MutexGuard};

use rand::Rng;

use crate::game::util::{load_rooms, to_buf};
use crate::interpreter::{CommandFunc, CommandKind, Interpreter};
use crate::item::{Describe, Holder, Item, ItemListTrait};
use crate::map::direction::MapDir;
use crate::map::door::{DoorState, GuardState, ObstacleState};
use crate::map::{coord::Coord, Locate, Room, RoomList, RoomListTrait, Space};
use crate::player::list::PlayerList;
use crate::player::{Player, Uuid};
use crate::text::message::{Audience, Broadcast, Message, Messenger, Msg};
use crate::text::Color::*;
use crate::text::{article, Wrap};
use crate::WriteResult;

mod broadcast;
mod commands;
mod item;
mod util;

type Error = Arc<crate::item::error::Error>;
pub type GameResult<T> = Result<T, Box<dyn StdError>>;

pub struct Game {
    players: PlayerList,
    rooms: RoomList,
    interpreter: Interpreter,
}

impl Game {
    pub fn new() -> GameResult<Self> {
        let (players, mut rooms) = (HashMap::new(), RoomList::default());

        load_rooms(&mut rooms)?;

        let mut interpreter = Interpreter::new();
        commands::fill_interpreter(&mut interpreter);

        Ok(Self {
            players: PlayerList(players),
            rooms,
            interpreter,
        })
    }

    pub fn interpret(&mut self, p: u128, s: &str) -> Option<String> {
        let (cmd, args) = Interpreter::process_string_command(s);

        let commands = self.interpreter.commands();
        let mut other_commands = commands.lock().ok()?;
        let mut cmd_func = other_commands.get_mut(&cmd)?.lock().ok()?;

        (*cmd_func)(self, p, &args)
    }

    pub fn add_player(&mut self, p: Player) {
        self.rooms.entry(p.loc()).or_default().add_player(&p);
        self.players.insert(p.uuid(), p);
    }

    pub fn remove_player<T: Uuid>(&mut self, p: T) -> Option<Player> {
        self.players.get_mut(&p.uuid())?.flush().ok()?;
        self.players.remove(&p.uuid())
    }

    pub fn broadcast<U>(&mut self, buf: U) -> io::Result<usize>
    where
        U: AsRef<[u8]>,
    {
        let mut res: usize = 0;
        for (_, p) in &mut *self.players {
            let mut s = String::from("\n\n");
            s.push_str(&String::from_utf8(buf.as_ref().to_owned()).unwrap());
            s.push_str("\n\n > ");
            res = p.write(s.as_bytes())?;
            p.flush()?;
        }
        Ok(res)
    }

    fn describe_room<P: Uuid>(&mut self, p: P) -> Option<String> {
        let loc = self.loc_of(p.uuid())?;

        let players = &mut self.players;
        let rooms = &self.rooms;
        let r = rooms.get(&loc)?;
        let exits = rooms.exits(loc);

        Some(r.display(p.uuid(), players, &exits))
    }

    fn describe_item<U>(&self, pid: U, handle: &str) -> Option<String>
    where
        U: Uuid,
    {
        let p = self.players.get(&pid.uuid())?;

        let loc = &p.loc();
        let room = self.rooms.get(loc)?;

        Some(if let Some(item) = room.get_item(handle) {
            let mut s = item.description().to_owned();
            if let Item::Container(lst) = item {
                s.push_str(&format!("\nthe {} is holding:\n", item.name()));
                s.push_str(&format!(
                    "{}",
                    Green(
                        lst.list()
                            .iter()
                            .map(|i| article(i.name()))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                ));
            }
            s
        } else {
            p.items().get(handle)?.description().to_owned()
        })
    }

    fn id_of(&self, name: &str) -> Option<u128> {
        self.players
            .iter()
            .find(|(_, p)| p.name() == name)
            .map(|(_, p)| p.uuid())
    }

    fn loc_of<P>(&self, p: P) -> Option<Coord>
    where
        P: Uuid,
    {
        Some(self.players.get(&p.uuid())?.loc())
    }

    fn name_of<P>(&self, p: P) -> Option<String>
    where
        P: Uuid,
    {
        Some(self.players.get(&p.uuid())?.name().into())
    }
    fn dir_func<U: Uuid>(&mut self, u: U, dir: MapDir) -> Option<String> {
        let u = u.uuid();
        let loc = self.loc_of(u)?;
        let name = self.name_of(u)?;

        let mut other_msg = None;

        let msg: Cow<'static, str> = match self.move_player(loc, u, dir) {
            Ok(_) => {
                other_msg = Some(format!("{} exits {}", name, dir));
                format!("you go {:?}\n\n{}", dir, self.describe_room(u)?).into()
            }
            Err(s) => {
                use crate::map::door::DoorState::*;
                match s {
                    None => "alas! you cannot go that way...".into(),
                    Closed => "a door blocks your way".into(),
                    Locked => "a door blocks your way".into(),
                    Open => "it's already open".into(),
                    MagicallySealed => {
                        "a door blocks your way. it's sealed with a mysterious force".into()
                    }
                    PermaLocked => {
                        "a door blocks your way. it's not going to budge, and there's no keyhole"
                            .into()
                    }
                    Guarded(s) => {
                        format!("{} blocks your way. they look pretty scary", article(&s)).into()
                    }
                }
            }
        };

        let rooms = &self.rooms;
        let others = rooms.player_ids(loc).except(u);
        let aud = Audience(u, &others);
        let msg = Msg {
            s: msg,
            o: other_msg.clone(),
        };
        self.send(aud, msg);
        if other_msg.is_none() {
            return Some("".into());
        }

        let next_room_aud = {
            let next_room = self.rooms.get(&loc.add(dir)?)?;
            Audience(0, Some(next_room.players_except(u)))
        };

        let for_others = format!("{} enters the room", name);
        let msg = "";
        self.send(
            next_room_aud,
            Msg {
                s: msg,
                o: Some(for_others),
            },
        );

        Some("".into())
    }

    fn describe_player<T>(&self, loc: Coord, pid: T, other: &str) -> Option<String>
    where
        T: Uuid,
    {
        let other_id = self.id_of_in(loc, other)?;
        let p = self.players.get(&other_id)?;

        let mut item_list = format!("{} is holding:", p.name());
        if p.items().len() > 0 {
            item_list.push('\n');
        }

        item_list.push_str(
            p.items()
                .iter()
                .map(|i| format!("{}", article(i.name())))
                .collect::<Vec<_>>()
                .join("\n")
                .as_str(),
        );

        Some(format!("{}{}", p.description(), item_list))
    }

    fn list_inventory<T: Uuid>(&self, u: T) -> Option<String> {
        let mut ret = String::new();
        ret.push_str("you are holding:\n");
        let items = self.players.get(&u.uuid())?.items();
        ret.push_str(
            items
                .iter()
                .map(|i| article(i.name()))
                .collect::<Vec<_>>()
                .join("\n")
                .as_str(),
        );

        Some(ret)
    }

    fn id_of_in(&self, loc: Coord, name: &str) -> Option<u128> {
        let rooms = &self.rooms;
        let players = &self.players;
        rooms.player_ids(loc).iter().find_map(|i| {
            let p = players.get(i)?;
            if p.name() == name {
                Some(p.uuid())
            } else {
                None
            }
        })
    }

    fn move_player(&mut self, loc: Coord, u: u128, dir: MapDir) -> Result<(), DoorState> {
        let next_coord = loc.add(dir);
        let rooms = &mut self.rooms;

        let src_room = rooms.get_mut(&loc)?;
        if let Some(door) = src_room.doors().get(&dir) {
            match door.state() {
                DoorState::None | DoorState::Open => (),
                s => return Err(s),
            }
        } else {
            let items = src_room.items();
            if let Some((d, g)) = items.iter().find_map(|i| {
                if let Item::Guard(d, g) = i {
                    Some((d, g))
                } else {
                    None
                }
            }) {
                if d == &dir && g.state() == GuardState::Closed {
                    return Err(DoorState::Guarded(g.name().to_owned()));
                }
            };
            src_room.players_mut().remove(&u);
        }

        rooms.get_mut(&next_coord?)?.players_mut().insert(u);
        let mut players = &mut self.players;
        players.get_mut(&u)?.set_loc(next_coord?);

        Ok(())
    }
}
