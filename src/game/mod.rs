mod commands;
mod item;
mod util;

use std::collections::HashMap;
use std::io;

use crate::game::MapDir::South;
use crate::interpreter::Interpreter;
use crate::item::{ItemTrait, ItemList2, ItemListTrait};
use crate::map::{coord::Coord, Locate, Room, RoomList, Space};
use crate::player::{Player, PlayerList, Uuid};
use crate::text::message::{Audience, Broadcast, Message, Messenger, Msg};
use crate::text::Color::*;
use crate::text::{article, Wrap};
use crate::WriteResult;

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::borrow::{BorrowMut, Cow};
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::sync::Arc;
use std::mem::take;
use crate::item::key::SkeletonKey;

type Error = Arc<crate::item::error::Error>;

pub struct Game {
    players: PlayerList,
    rooms: RoomList,
    interpreter: Interpreter,
}

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum MapDir {
    North,
    South,
    East,
    West,
    Up,
    Down,
    NorthEast,
    // etc
}

impl Default for MapDir {
    fn default() -> Self {
        South
    }
}

impl MapDir {
    const ALL_DIRS: [MapDir; 7] = [
        MapDir::North,
        MapDir::South,
        MapDir::East,
        MapDir::West,
        MapDir::Up,
        MapDir::Down,
        MapDir::NorthEast,
    ];

    pub fn all() -> &'static [Self] {
        &Self::ALL_DIRS
    }
}

impl std::fmt::Debug for MapDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use MapDir::*;

        write!(
            f,
            "{}",
            match self {
                North => "north",
                South => "south",
                East => "east",
                West => "west",
                Up => "up",
                Down => "down",
                NorthEast => "northeast",
            }
        )
    }
}

impl Display for MapDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use MapDir::*;

        write!(
            f,
            "{}",
            match self {
                North => "n",
                South => "s",
                East => "e",
                West => "w",
                Up => "u",
                Down => "d",
                NorthEast => "ne",
            }
        )
    }
}

impl<T> Broadcast for T
where
    T: BorrowMut<Game>,
{
    fn send<A, M>(&mut self, audience: A, message: M) -> Vec<WriteResult>
    where
        A: Messenger,
        M: Message,
    {
        let g = self.borrow_mut();
        let mut v = vec![];
        let self_id = audience.id().unwrap_or_default();
        let other_ids = audience.others().unwrap_or_default();

        let self_msg = message.to_self();
        let other_msg = message.to_others();

        if let Some(p) = g.players.get_mut(&self_id) {
            let self_msg = self_msg.wrap(90);
            v.push(p.write(to_buf(self_msg).as_slice()));
        }

        if let Some(msg) = other_msg {
            let msg = msg.wrap(90);
            for id in other_ids {
                if let Some(p) = g.players.get_mut(&id) {
                    v.push(p.write(to_buf(&msg).as_slice()));
                }
            }
        }

        v
    }
}

fn to_buf<T: AsRef<str>>(msg: T) -> Vec<u8> {
    let buf = msg.as_ref().as_bytes();
    let mut b = vec![];
    b.extend_from_slice(b"\n".as_ref());
    b.extend_from_slice(buf.as_ref());
    b.extend_from_slice(b"\n\n > ".as_ref());
    b
}

impl Game {
    pub fn new() -> Self {
        let (players, mut rooms) = (HashMap::new(), RoomList::default());
        let bytes = include_bytes!("../../data/map.cbor");
        let v: Vec<Room> = serde_cbor::from_slice(bytes).unwrap_or_default();
        let p = Player::new("billy");

        let mut count = 0;
        for mut r in v {
            r.init();
            if count == 0 {
                let key = SkeletonKey {
                    handle: Handle(Vec::new()),
                };
                r.items_mut().insert(key)
            }
            rooms.insert(r.loc(), r);
            count += 1;
        }

        let mut interpreter = Interpreter::new();
        commands::fill_interpreter(&mut interpreter);

        let mut ret = Self {
            players: PlayerList(players),
            rooms,
            interpreter,
        };

        ret.add_player(p);
        ret
    }

    fn describe_room<P: Uuid>(&mut self, p: P) -> Option<String> {
        let loc = self.loc_of(p.uuid())?;

        let players = &mut self.players;
        let rooms = &self.rooms;
        let r = loc.room(rooms)?;

        Some(r.display(p.uuid(), players, rooms))
    }

    fn id_of(&self, name: &str) -> Option<u128> {
        self.players
            .iter()
            .find(|(_, p)| p.name() == name)
            .map(|(_, p)| p.uuid())
    }

    pub fn interpret(&mut self, p: u128, s: &str) -> Option<String> {
        let mut words = s.split_whitespace();
        let cmd_str = words.next().unwrap_or_default();
        let args: Vec<&str> = words.collect();
        let cmd = Interpreter::resolve_str(cmd_str);

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

    fn get_player(&self, u: u128) -> Option<&Player> {
        self.players.get(&u)
    }

    fn describe_item<U>(&self, pid: U, handle: &str) -> Option<&str>
    where
        U: Uuid,
    {
        let p = self.get_player(pid.uuid())?;

        let loc = &p.loc();
        let room = self.rooms.get(loc)?;

        Some(if let Some(item) = room.get_item(handle) {
            &item.description()
        } else {
            p.items().get(handle)?.description()
        })
    }

    fn dir_func<U: Uuid>(&mut self, u: U, dir: MapDir) -> Option<String> {
        let u = u.uuid();
        let loc = self.loc_of(u)?;
        let name = self.name_of(u)?;

        let mut other_msg = None;

        let msg: Cow<'static, str> = match loc.move_player(self, u, dir.clone()) {
            Ok(_) => {
                other_msg = Some(format!("{} exits {}", name, dir));
                format!("you go {:?}\n\n{}", dir, self.describe_room(u)?).into()
            }
            Err(s) => {
                use crate::map::door::DoorState::*;
                let err_msg = match s {
                    None => "alas! you cannot go that way...",
                    Closed => "a door blocks your way",
                    Locked => "a door blocks your way",
                    Open => "it's already open",
                    MagicallySealed => {
                        "a door blocks your way. it's sealed with a mysterious force"
                    }
                    PermaLocked => {
                        "a door blocks your way. it's not going to budge, and there's no keyhole"
                    }
                };
                self.send(u, err_msg);
                return Some("".into());
            }
        };

        let others = loc.player_ids(&self)?;
        let aud = Audience(u, &others);
        let msg = Msg {
            s: msg,
            o: other_msg,
        };
        self.send(aud, msg);

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

    fn describe_player<T>(&self, pid: T, other: &str) -> Option<String>
    where
        T: Uuid,
    {
        let p = self.loc_of(pid)?.player_by_name(self, other)?;

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
}

fn random_insult() -> String {
    match rand::thread_rng().gen_range(1, 6) {
        1 => "dude wtf",
        2 => "i think you should leave",
        3 => "i'll have to ask my lawyer about that",
        4 => "that's ... uncommon",
        _ => "that's an interesting theory... but will it hold up in the laboratory?",
    }
    .to_owned()
}
