use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io;
use std::io::Write;

use rand::Rng;

use crate::error::EnnuiError;
use crate::error::EnnuiError::Fatal;
use crate::game::util::load_rooms;
use crate::interpreter::{CommandMessage, Interpreter};
use crate::item::list::Holder;
use crate::item::list::ListTrait;
use crate::item::{Describe, Item};
use crate::map::direction::MapDir;
use crate::map::door::{DoorState, GuardState, ObstacleState};
use crate::map::list::{RoomList, RoomListTrait};
use crate::map::{coord::Coord, Locate, Room, Space};
use crate::player::list::{PlayerIdList, PlayerList};
use crate::player::{Player, Uuid};
use crate::text::article;
use crate::text::message::{Audience, Broadcast, Message, Messenger, Msg};
use crate::text::Color::*;

mod broadcast;
mod commands;
mod item;
mod util;

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

    pub fn interpret(&mut self, p: u128, s: &str) -> Result<CommandMessage, EnnuiError> {
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

    pub fn announce_player(&mut self, u: u128) {
        let (name, players) = {
            (
                self.players
                    .get(&u)
                    .unwrap_or(&Player::default())
                    .name()
                    .to_owned(),
                self.players.to_id_list().except(u),
            )
        };
        self.send(&players, &format!("{} has joined the game.", name));
    }

    pub fn remove_player<T: Uuid>(&mut self, p: T) -> Option<Player> {
        self.players.get_mut(&p.uuid())?.flush().ok()?;
        self.players.remove(&p.uuid())
    }

    pub fn broadcast<U>(&mut self, buf: U) -> io::Result<usize>
    where
        U: AsRef<[u8]>,
    {
        let mut res = 0_usize;
        for (_, p) in &mut *self.players {
            let mut s = String::from("\n\n");
            s.push_str(&String::from_utf8(buf.as_ref().to_owned()).unwrap());
            s.push_str("\n\n > ");
            res = p.write(s.as_bytes())?;
            p.flush()?;
        }
        Ok(res)
    }

    pub fn players_in(&mut self, loc: Coord) -> Cow<PlayerIdList> {
        match self.rooms.get(&loc) {
            Some(r) => Cow::Borrowed(r.players()),
            None => Cow::Owned(PlayerIdList::default()),
        }
    }

    pub fn players_mut(&mut self) -> &mut PlayerList {
        &mut self.players
    }

    pub fn set_player_name(&mut self, u: u128, name: &str) -> Result<(), EnnuiError> {
        Ok(self
            .players
            .get_mut(&u)
            .ok_or(EnnuiError::Fatal(
                "CANNOT SET NAME: Player Not Found".to_owned(),
            ))?
            .set_name(name))
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
            p.items().get_item(handle)?.description().to_owned()
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

    fn dir_func<U: Uuid>(
        &mut self,
        u: U,
        dir: MapDir,
    ) -> Result<(Box<dyn Messenger>, Box<dyn Message>), EnnuiError> {
        let u = u.uuid();
        let loc = self.loc_of(u).ok_or(Fatal(format!("player not found")))?;
        let name = self.name_of(u).ok_or(Fatal(format!("player not found")))?;

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
        self.send(&aud, &msg);
        if other_msg.is_none() {
            return message(0, "");
        }

        let next_room_aud = {
            let next_room = self.rooms.get(&loc.add(dir)?)?;
            Audience(0, next_room.players_except(u))
        };

        let for_others = format!("{} enters the room", name);
        let msg = "";
        self.send(
            &next_room_aud,
            &Msg {
                s: msg,
                o: Some(for_others),
            },
        );

        message(0, "")
    }

    fn describe_player<T>(&self, loc: Coord, _pid: T, other: &str) -> Option<String>
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
        let players = &mut self.players;

        let src_room = rooms.get_mut(&loc)?;

        Game::check_doors(dir, src_room)?;
        Self::check_guard(dir, src_room)?;

        src_room.players_mut().remove(&u);

        Self::do_player_move(players, u, next_coord, rooms)
    }

    fn do_player_move(
        players: &mut PlayerList,
        u: u128,
        next_coord: Option<Coord>,
        rooms: &mut RoomList,
    ) -> Result<(), DoorState> {
        let next_coord = next_coord.ok_or(DoorState::None)?;

        rooms
            .get_mut(&next_coord)
            .ok_or(DoorState::None)?
            .players_mut()
            .insert(u);

        players
            .get_mut(&u)
            .ok_or(DoorState::None)?
            .set_loc(next_coord);

        Ok(())
    }

    fn check_doors(dir: MapDir, src_room: &mut Room) -> Result<(), DoorState> {
        if let Some(door) = src_room.doors().get(&dir) {
            match door.state() {
                DoorState::None | DoorState::Open => (),
                s => return Err(s),
            }
        }

        Ok(())
    }

    fn check_guard(dir: MapDir, src_room: &Room) -> Result<(), DoorState> {
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
        }

        Ok(())
    }

    #[cfg(test)]
    pub fn get_room(&self, loc: &Coord) -> Option<&Room> {
        self.rooms.get(loc)
    }
}

pub fn message<A: 'static, M: 'static>(
    aud: A,
    msg: M,
) -> Result<(Box<dyn Messenger>, Box<dyn Message>), EnnuiError>
where
    A: Messenger,
    M: Message,
{
    Ok((Box::new(aud), Box::new(msg)))
}
