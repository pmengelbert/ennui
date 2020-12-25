mod commands;
mod util;

use std::collections::HashMap;
use std::io;

use crate::interpreter::Interpreter;
use crate::item::Holder;
use crate::map::{Coord, Locate, Room, RoomList};
use crate::player::{Player, PlayerList, Uuid};
use crate::text::article;
use crate::text::Color::*;
use crate::{mapdata, PassFail, WriteResult};

use rand::Rng;
use std::fmt::{Display, Formatter};
use std::io::Write;
use serde::private::de::TagContentOtherField;
use crate::text::message::{Broadcast, Broadcast2, Messenger, Message};

pub struct Game {
    players: PlayerList,
    rooms: RoomList,
    interpreter: Interpreter,
}

enum Direction {
    Take,
    Give,
    Drop,
    Wear,
    Remove,
}

#[derive(Copy, Clone)]
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
    T: AsMut<Game>,
{
    fn send_to_player<U, S>(&mut self, u: U, msg: S) -> WriteResult
    where
        U: Uuid,
        S: AsRef<str>,
    {
        let g = self.as_mut();
        let buf = msg.as_ref().as_bytes();
        match g.players.get_mut(&u.uuid()) {
            Some(p) => {
                let mut b = vec![];
                b.extend_from_slice(b"\n\n".as_ref());
                b.extend_from_slice(buf.as_ref());
                b.extend_from_slice(b"\n\n > ".as_ref());
                let res = p.write(b.as_ref())?;
                p.flush()?;
                Ok(res)
            }
            None => Err(std::io::ErrorKind::AddrNotAvailable.into()),
        }
    }
}

impl<T> Broadcast2 for T
where
    T: AsMut<Game>
{
    fn send<A, M>(&mut self, audience: A, message: M) -> Vec<WriteResult> where A: Messenger, M: Message {
        let mut g = self.as_mut();
        let mut v = vec![];
        let self_id = audience.id().unwrap_or_default();
        let other_ids = audience.others().unwrap_or_default();

        let self_msg = message.to_self();
        let other_msg = message.to_others();

        if let Some(p) = g.players.get_mut(&self_id) {
            v.push(p.write(self_msg.as_bytes()));
        }

        if let Some(msg) = other_msg {
            for id in other_ids {
                if let Some(p) = g.players.get_mut(&id) {
                    v.push(p.write(msg.as_bytes()));
                }
            }
        }

        v
    }
}

fn to_buf<T: AsRef<str>>(msg: T) -> Vec<u8> {
    let buf = msg.as_ref().as_bytes();
    let mut b = vec![];
    b.extend_from_slice(b"\n\n".as_ref());
    b.extend_from_slice(buf.as_ref());
    b.extend_from_slice(b"\n\n > ".as_ref());
    b
}

impl Game {
    pub fn new() -> Self {
        let (players, mut rooms) = (HashMap::new(), RoomList::default());
        let v: Vec<Room> = serde_cbor::from_slice(mapdata::MAP.as_ref()).unwrap_or_default();
        let p = Player::new("billy");

        for r in v {
            rooms.insert(r.loc(), r);
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
        let loc = self.loc_of(u.uuid())?;

        let u = u.uuid();
        Some(match loc.move_player(self, u, dir.clone()) {
            Ok(_) => format!("you go {:?}\n\n{}", dir, self.describe_room(u)?),
            Err(_) => format!("alas! you cannot go that way..."),
        })
    }

    fn describe_player<T>(&self, pid: T, other: &str) -> Option<String>
    where
        T: Uuid,
    {
        let p = self.loc_of(pid)?.player_by_name(self, other)?;

        let item_list = match p.items().len() {
            0 => "".to_owned(),
            _ => format!(
                "\n{} is holding:\n{}",
                p.name(),
                p.items()
                    .iter()
                    .map(|i| format!(" --> {}", article(i.name())))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        };

        Some(format!("{}{}", p.description().to_owned(), item_list))
    }

    fn list_inventory<T: Uuid>(&self, u: T) -> Option<String> {
        let mut ret = String::new();
        ret.push_str("you are holding:\n");
        let items = self.players.get(&u.uuid())?.items();
        let ret = items
            .iter()
            .map(|i| {
                let name = i.name();
                format!("{}", article(name))
            })
            .collect::<Vec<_>>()
            .join("\n");

        Some(ret)
    }

    fn loc_of<P>(&self, p: P) -> Option<Coord>
    where
        P: Uuid,
    {
        Some(self.players.get(&p.uuid())?.loc())
    }

    #[allow(dead_code)]
    fn name_of<P>(&self, p: P) -> Option<&str>
    where
        P: Uuid,
    {
        Some(self.players.get(&p.uuid())?.name())
    }

    fn transfer<T>(&mut self, u: T, other: Option<&str>, dir: Direction, handle: &str) -> PassFail
    where
        T: Uuid,
    {
        use Direction::*;
        let uuid = &u.uuid();
        let loc = &self.loc_of(u)?;

        let rooms = &mut self.rooms;
        let players = &mut self.players;
        match dir {
            Take => {
                rooms
                    .get_mut(loc)?
                    .transfer(players.get_mut(uuid)?, handle)
                    .ok()?;
            }
            Drop => {
                players
                    .get_mut(uuid)?
                    .transfer(rooms.get_mut(loc)?, handle)
                    .ok()?;
            }
            Give => {
                let item = players.get_mut(uuid)?.remove_item(handle)?;
                loc.player_by_name_mut(self, other?)?.give_item(item);
            }
            Wear => {
                let (items, clothing) = players.get_mut(uuid)?.all_items_mut();
                items.transfer(clothing, handle)?;
            }
            Remove => {
                let (items, clothing) = players.get_mut(uuid)?.all_items_mut();
                clothing.transfer(items, handle)?;
            }
        }

        Ok(())
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

// #[cfg(test)]
// mod game_test {
//     use super::*;
//     use std::borrow::{Borrow, BorrowMut};
//     use std::sync::RwLock;
//     use std::mem::take;
//
//     struct Thing {
//         inner: HashMap<usize, RwLock<Room>>,
//     }
//
//     impl Thing {
//         fn new() -> Self {
//             let mut inner = HashMap::new();
//             let mut g = Game::new();
//             let r1 = take(g.rooms.get_mut(&Coord(0, 0)).unwrap());
//             let r2 = take(g.rooms.get_mut(&Coord(0, 1)).unwrap());
//             inner.insert(0, RwLock::new(r1));
//             inner.insert(1, RwLock::new(r2));
//
//             Thing { inner }
//         }
//
//         // DOES NOT WORK
//         fn thing(&mut self) -> Option<String> {
//             let r2 = self.inner.get_mut(&0).unwrap().read().unwrap().borrow();
//             let mut r1 = self.inner.get(&1).unwrap().write().unwrap().borrow_mut();
//             r1.add_player(&(7 as u128));
//             let x = r2.get_item("codpiece").unwrap();
//             Some(format!("{} {}", r1.players().len(), x.name()))
//         }
//     }
//
//     #[test]
//     fn test_interior_mutability() {
//         assert_eq!(Thing::new().thing(), Some("1 codpiece".to_owned()));
//     }
//
//     fn new_game() -> Game {
//         let mut g = Game::new();
//         let p = Player::new("peter");
//
//         let uuid = p.uuid();
//         g.add_player(p);
//         g
//     }
//
//     #[test]
//     fn game_test_display_room() {
//         let p = Player::new("lol");
//         let uuid = p.uuid();
//         let q = Player::new("billy");
//         let pp = Player::new("mindy");
//
//         let mut r = Room::new("the room", None);
//         let mut g = Game::new();
//         for player in vec![p, q, pp] {
//             r.add_player(&player);
//             g.players.insert(player.uuid(), player);
//         }
//         g.rooms.insert(Coord(0, 0), r);
//
//         println!("{}", g.describe_room(uuid)?);
//     }
//
//     #[test]
//     fn game_test_interpreter() {
//         let mut g = Game::new();
//         let mut r = Room::new("yo", None);
//         let p = Player::new("lol");
//         r.add_player(&p);
//         let uuid = p.uuid();
//         g.add_player(p);
//
//         assert!(g.interpret(uuid, "look").is_some());
//     }
// }
