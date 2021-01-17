use std::borrow::Cow;
use std::collections::HashMap;
use std::error::{Error as StdError, Error};

use std::io::Write;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use rand::Rng;

use crate::error::EnnuiError;
use crate::error::EnnuiError::{Fatal, Lesser};
use crate::fight::FightMessage;
use crate::game::util::load_rooms;
use crate::interpreter::CommandQuality::{Awake, Motion};
use crate::interpreter::{CommandKind, CommandMessage, Interpreter};
use crate::item::list::ListTrait;
use crate::item::list::{Holder, ItemListTrout};

use crate::item::{Attribute, Describe, Item};
use crate::map::direction::MapDir;
use crate::map::door::{DoorState, GuardState, ObstacleState};
use crate::map::list::{RoomList, RoomListTrait};
use crate::map::{coord::Coord, Locate, Room, Space};
use crate::player::list::{PlayerIdList, PlayerIdListTrait, PlayerList, PlayerListTrait};
use crate::player::PlayerStatus::{Asleep, Dead, Sitting};
use crate::player::{PlayerType, Uuid};
use crate::text::article;
use crate::text::message::{
    Audience, Broadcast, FightAudience, Message, MessageFormat, Messenger, Msg,
};
use crate::text::Color::{Green, Magenta};
use std::fmt::Debug;
use std::mem::take;

mod broadcast;
mod commands;
mod item;
mod util;

pub type GameResult<T> = Result<T, Box<dyn StdError>>;
pub type GameOutput = (Box<dyn Messenger>, Box<dyn Message>);

pub trait NpcInit {
    fn init_npcs(&self, npcs: Vec<PlayerType>) -> Result<(), EnnuiError>;
}

impl NpcInit for Arc<Mutex<Game>> {
    fn init_npcs(&self, npcs: Vec<PlayerType>) -> Result<(), EnnuiError> {
        for mut possible_npc in npcs.into_iter() {
            if let PlayerType::Npc(ref mut npc) = &mut possible_npc {
                npc.init(self.clone());
            }
            self.lock().unwrap().add_player(possible_npc);
        }
        Ok(())
    }
}

pub struct Game {
    players: PlayerList,
    rooms: RoomList,
    interpreter: Interpreter,
    fight_sender: Option<Sender<(FightAudience, FightMessage)>>,
}

impl Game {
    pub fn new() -> GameResult<Self> {
        let (players, mut rooms) = (HashMap::new(), RoomList::default());

        load_rooms(&mut rooms)?;

        let mut interpreter = Interpreter::new();
        commands::fill_interpreter(&mut interpreter);

        let mut g = Self {
            players,
            rooms,
            interpreter,
            fight_sender: None,
        };

        Ok(g)
    }

    pub fn set_fight_sender(&mut self, sender: Sender<(FightAudience, FightMessage)>) {
        self.fight_sender = Some(sender);
    }

    pub fn interpret(&mut self, p: u128, s: &str) -> Result<CommandMessage, EnnuiError> {
        let s = s.to_lowercase();
        eprintln!("executing command '{}' for player {}", s, p);
        let (cmd, args) = Interpreter::process_string_command(&s);

        let commands = self.interpreter.commands();
        let other_commands = commands.lock().ok()?;
        let cmd_func = other_commands.get(&cmd)?.lock().ok()?;

        if let Some(msg) = self.verify_status(cmd, p)? {
            return Ok(msg);
        }

        (*cmd_func)(self, p, &args)
    }

    pub fn interpreter(&mut self) -> &mut Interpreter {
        &mut self.interpreter
    }

    pub fn add_player(&mut self, p: PlayerType) {
        self.rooms.entry(p.loc()).or_default().add_player(&p);
        self.players.insert(p.uuid(), Arc::new(Mutex::new(p)));
    }

    pub fn announce_player(&mut self, u: u128) -> Result<(), EnnuiError> {
        let (name, players) = {
            (
                self.get_player(u)?.name(),
                self.players.to_id_list().except(u),
            )
        };
        self.send(
            &players,
            &format!("{} has joined the game.", name).custom_padded("\n\n", "\n\n > "),
        );
        Ok(())
    }

    pub fn remove_player<T: Uuid>(&mut self, p: T) -> Option<Arc<Mutex<PlayerType>>> {
        let mut name = String::new();
        let mut messages = vec![];
        let player = self.get_player(p.uuid())?;
        let mut player = player.lock().unwrap();
        name.push_str(&player.name());
        let room = self.get_room_mut(player.loc())?;
        let mut items = take(player.items_mut());

        let aud = room.players().except(p.uuid());
        for item in items.iter_mut() {
            let owned_item = take(item);
            messages.push(format!("{} drops {}", name, article(&owned_item.name())));
            if room.insert_item(owned_item).is_err() {
                print_err(lesser("Unable to drop item for player"));
            };
        }

        player.flush().ok()?;
        player.drop_stream();

        let res = self.players.remove(&p.uuid());

        for message in messages {
            self.send(&aud, &message.custom_padded("\n", ""));
        }

        self.send(
            &self.players.to_id_list(),
            &format!("{} has left the game", name).padded(),
        );
        res
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
        self.get_player(u)?.lock().unwrap().set_name(name);
        Ok(())
    }

    pub fn clone_fight_sender(&self) -> Result<Sender<(FightAudience, FightMessage)>, EnnuiError> {
        Ok(self
            .fight_sender
            .as_ref()
            .ok_or_else(|| fatal("ERROR: UNABLE TO CLONE SENDER"))?
            .clone())
    }

    pub fn get_room(&self, loc: Coord) -> Result<&Room, EnnuiError> {
        self.rooms
            .get(&loc)
            .ok_or_else(|| fatal("UNABLE TO FIND ROOM"))
    }

    pub fn get_room_mut(&mut self, loc: Coord) -> Result<&mut Room, EnnuiError> {
        self.rooms
            .get_mut(&loc)
            .ok_or_else(|| fatal("UNABLE TO FIND ROOM"))
    }

    fn describe_room<P: Uuid>(&mut self, p: P) -> Result<String, EnnuiError> {
        eprintln!("[{}]: describe_room", "SUCCESS".color(Green));
        let loc = self.loc_of(p.uuid())?;
        eprintln!("[{}]: got uuid", "SUCCESS".color(Green));

        let rooms = &self.rooms;
        let r = rooms.get(&loc)?;
        let player_list_string = r.players().except(p.uuid()).display(&self.players);
        eprintln!("[{}]: got room", "SUCCESS".color(Green));
        let exits = Room::exit_display(&rooms.exits(loc));

        let mut room_string = r.display();
        if !player_list_string.is_empty() {
            room_string.push_str(&player_list_string)
        }
        room_string.push_str(&exits);
        Ok(room_string)
    }

    fn describe_item<U>(&self, pid: U, handle: &str) -> Option<String>
    where
        U: Uuid,
    {
        let p = self.players.get(&pid.uuid())?;

        let loc = &p.lock().unwrap().loc();
        let room = self.rooms.get(loc)?;

        Some(if let Some(item) = room.get_item(handle) {
            let mut s = item.description();
            if let Item::Container(lst) = item {
                s.push_str(&format!("\nthe {} is holding:\n", item.name()));
                s.push_str(
                    &lst.list()
                        .iter()
                        .map(|i| article(&i.name()))
                        .collect::<Vec<_>>()
                        .join("\n")
                        .color(Green),
                );
            }
            s
        } else {
            p.lock().unwrap().items().get_item(handle)?.description()
        })
    }

    fn id_of(&self, name: &str) -> Option<u128> {
        self.players
            .iter()
            .find(|(_, p)| p.lock().unwrap().handle() == name)
            .map(|(_, p)| p.lock().unwrap().uuid())
    }

    fn verify_status(
        &self,
        cmd: CommandKind,
        u: u128,
    ) -> Result<Option<CommandMessage>, EnnuiError> {
        let p = self.get_player(u)?;
        let p = p.lock().unwrap();
        let cnv = |m: Result<CommandMessage, EnnuiError>| m.map(Some);

        if p.is(Dead) && cmd != CommandKind::Quit {
            return cnv(message(u, "oh boy. you can't move. you're dead."));
        }

        if p.is(Asleep) && cmd.is(Awake) {
            return cnv(message(u, "you're asleep. why not just sleep?"));
        }

        if p.is(Sitting) && cmd.is(Motion) {
            return cnv(message(u, "why don't you try standing up first?"));
        }

        Ok(None)
    }

    fn get_player(&self, p: u128) -> Result<Arc<Mutex<PlayerType>>, EnnuiError> {
        Ok(self
            .players
            .get(&p)
            .ok_or_else(|| fatal("PLAYER NOT FOUND"))?
            .clone())
    }

    fn loc_of<P>(&self, p: P) -> Result<Coord, EnnuiError>
    where
        P: Uuid,
    {
        Ok(self.get_player(p.uuid())?.loc())
    }

    fn name_of<P>(&self, p: P) -> Result<String, EnnuiError>
    where
        P: Uuid,
    {
        Ok(self.get_player(p.uuid())?.name())
    }

    fn dir_func<U: Uuid>(&mut self, u: U, dir: MapDir) -> Result<GameOutput, EnnuiError> {
        let u = u.uuid();
        let loc = self.loc_of(u)?;
        let name = self.name_of(u)?;

        let mut other_msg = None;
        let mut terminate = None;

        let msg: Cow<'static, str> = match self.move_player(loc, u, dir) {
            Ok(_) => {
                other_msg = Some(format!("{} exits {}", name, dir));
                format!("you go {:?}\n\n{}", dir, self.describe_room(u)?).into()
            }
            Err(s) => {
                use crate::map::door::DoorState::*;
                terminate = Some(());
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
        let aud = Audience(u, others);
        let msg = String::from(msg);
        let msg = FightMessage {
            s: msg.into(),
            obj: None,
            oth: other_msg.clone().map(|s| s.padded().into()),
        };

        let return_msg = message(aud, msg);
        if terminate.is_some() {
            return return_msg;
        }

        let next_room_aud = {
            if let Some(next_room) = self.rooms.get(&loc.add(dir)?) {
                Audience(0, next_room.players_except(u))
            } else {
                return return_msg;
            }
        };

        let for_others = format!("{} enters the room", name);
        let msg = "";
        self.send(
            &next_room_aud,
            &Msg {
                s: msg,
                o: Some(for_others.padded()),
            },
        );

        return_msg
    }

    fn describe_player<T>(&self, loc: Coord, _pid: T, other: &str) -> Option<String>
    where
        T: Uuid,
    {
        let other_id = self.id_of_in(loc, other)?;
        let p = self.players.get(&other_id)?;
        let p = p.lock().unwrap();

        let description = p.description();

        let item_list_title = format!("\n{} is holding:", p.name());
        let mut item_list = String::new();
        for item in p.items().iter() {
            item_list.push('\n');
            item_list.push_str(&article(&item.name()));
        }

        Some(format!(
            "{}{}{}",
            description,
            item_list_title,
            item_list.color(Green)
        ))
    }

    fn list_inventory<T: Uuid>(&self, u: T) -> Result<String, EnnuiError> {
        let mut s = String::new();
        s.push_str("you are holding:");

        let player = self.get_player(u.uuid())?;

        for item in player.lock().unwrap().items().iter() {
            s.push('\n');
            s.push_str(&article(&item.name()).color(Green))
        }

        Ok(s)
    }

    fn id_of_in(&self, loc: Coord, name: &str) -> Option<u128> {
        let rooms = &self.rooms;
        let players = &self.players;

        rooms.player_ids(loc).iter().find_map(|i| {
            let p = players.get(i)?;
            if p.handle() == name {
                Some(p.lock().unwrap().uuid())
            } else {
                None
            }
        })
    }

    fn move_player(&mut self, loc: Coord, u: u128, dir: MapDir) -> Result<(), DoorState> {
        let next_coord = loc.add(dir);

        self.check_doors(loc, next_coord, dir)?;

        let rooms = &mut self.rooms;
        let players = &mut self.players;
        let src_room = rooms.get_mut(&loc)?;

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

        let mut p = players.get_mut(&u).ok_or(DoorState::None)?.lock().unwrap();
        p.set_loc(next_coord);

        if let Err(e) = p.leave_fight() {
            eprintln!("ERROR: {:?}", e);
        };

        Ok(())
    }

    fn check_doors(&self, loc: Coord, next_coord: Option<Coord>, dir: MapDir) -> Result<(), DoorState> {
        {
            let src_room = self.rooms.get(&loc)?;
            if let Some(door) = src_room.doors().get(&dir) {
                match door.state() {
                    DoorState::None | DoorState::Open => (),
                    s => return Err(s),
                }
            }
        }

        {
            let dst_room = self.rooms.get(&next_coord?)?;
            if let Some(door) = dst_room.doors().get(&dir.opposite()) {
                match door.state() {
                    DoorState::None | DoorState::Open => (),
                    s => return Err(s),
                }
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
                return Err(DoorState::Guarded(g.name()));
            }
        }

        Ok(())
    }
}

pub fn message<A: 'static, M: 'static>(aud: A, msg: M) -> Result<GameOutput, EnnuiError>
where
    A: Messenger,
    M: Message,
{
    let obj = aud
        .object()
        .map(|_| msg.to_object().unwrap_or_default().padded().into());
    let oth = msg.to_others().map(|m| m.padded().into());
    let msg = FightMessage {
        s: msg.to_self().padded().into(),
        obj,
        oth,
    };

    Ok((Box::new(aud), Box::new(msg)))
}

fn fatal(s: &str) -> EnnuiError {
    Fatal(s.to_owned())
}

fn lesser(s: &str) -> EnnuiError {
    Lesser(s.to_owned())
}

fn print_err<T: Error + Debug>(err: T) {
    eprintln!("[{}]: {:?}", "ERROR".color(Magenta), err)
}
