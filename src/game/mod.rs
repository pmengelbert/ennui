use std::collections::HashMap;
use std::mem::{swap, take};
use std::option::NoneError;
use std::{io, process};

use crate::interpreter::Interpreter;
use crate::item::{Holder, Item, ItemKind, ItemList};
use crate::map::{Coord, Locate, Room, RoomList};
use crate::player::{Player, PlayerList, Uuid};
use crate::text::{Color::*, Wrap};

use crate::PassFail;
use rand::Rng;
use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::ptr::replace;

impl AsRef<Game> for Game {
    fn as_ref(&self) -> &Game {
        self
    }
}

impl AsMut<Game> for Game {
    fn as_mut(&mut self) -> &mut Game {
        self
    }
}

pub trait Provider<T> {
    fn provide(&self) -> &T;
    fn provide_mut(&mut self) -> &mut T;
}

impl<T> Provider<PlayerList> for T
where
    T: AsRef<Game> + AsMut<Game>,
{
    fn provide(&self) -> &PlayerList {
        &self.as_ref().players
    }

    fn provide_mut(&mut self) -> &mut PlayerList {
        &mut self.as_mut().players
    }
}

impl<T> Provider<RoomList> for T
where
    T: AsRef<Game> + AsMut<Game>,
{
    fn provide(&self) -> &RoomList {
        &self.as_ref().rooms
    }

    fn provide_mut(&mut self) -> &mut RoomList {
        &mut self.as_mut().rooms
    }
}

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

#[derive(Clone, Debug)]
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

impl Display for MapDir {
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

impl Game {
    pub fn new() -> Self {
        let (players, mut rooms) = (HashMap::new(), RoomList::default());
        let desc = r#"You are at the Temple Yard of Dragonia. Beautiful marble stairs lead up to the Temple of Dragonia. You feel small as you stare up the huge pillars making the entrance to the temple. This place serves as a sanctuary where the people of the city can come and seek refuge, and rest their tired bones. Just north of here is the common square, and the temple opens to the south."#;
        let mut r = Room::new("the living room", Some(desc), Coord(0, 0));
        let mut r2 = Room::new(
            "the other room",
            Some(&desc.chars().rev().collect::<String>()),
            Coord(0, 1),
        );
        let mut p = Player::new("billy");
        p.set_description("this guy is a silly billy, don't you think?");
        r.add_player(&p);
        let i = ItemKind::Clothing(Item::new(
            "codpiece",
            Some("a beautifully decorated codpiece. truly a wonder"),
            "codpiece",
        ));
        r.add_item(i);
        rooms.insert(r.loc(), r);
        rooms.insert(r2.loc(), r2);
        let mut interpreter = Interpreter::new();
        fill_interpreter(&mut interpreter);

        let mut ret = Self {
            players: PlayerList(players),
            rooms,
            interpreter,
        };

        ret.add_player(p);
        ret
    }

    fn describe_room<P: Uuid>(&mut self, p: P) -> Option<String> {
        let mut ret = "".to_owned();
        let loc = self.loc_of(p.uuid())?;

        let players = &mut self.players;
        let rooms = &self.rooms;
        let r = loc.room(rooms)?;

        ret = r.display(p.uuid(), players);

        Some(ret)
    }

    /// `interpret` will interpret a command (`s`) given by the player `p`, returning
    /// the response to the command.
    pub fn interpret(&mut self, p: u128, s: &str) -> Option<String> {
        let mut interpreter = take(&mut self.interpreter);

        let mut ret = None;
        with_cleanup!(('interpreter) {
            ret = Some(goto_cleanup_on_fail!(interpreter.interpret(self, p, s), 'interpreter));
        } 'cleanup: {
            self.interpreter = interpreter;
        });

        if ret.is_none() {
            let quit_string = "quit";
            if !quit_string.starts_with(&s[..min(s.len(), quit_string.len())]) {
                ret = Some(random_insult())
            }
        }

        ret
    }

    pub fn add_player(&mut self, p: Player) {
        self.rooms.entry(p.loc()).or_default().add_player(&p);
        self.players.insert(p.uuid(), p);
    }

    pub fn remove_player<T: Uuid>(&mut self, p: T) -> Option<Player> {
        self.players.get_mut(&p.uuid())?.flush();
        self.players.remove(&p.uuid())
    }

    pub fn send_to_player<P, U>(&mut self, p: P, buf: U) -> std::io::Result<usize>
    where
        P: Uuid,
        U: AsRef<[u8]>,
    {
        match self.players.get_mut(&p.uuid()) {
            Some(p) => {
                let res = p.write(buf.as_ref())?;
                p.flush()?;
                Ok(res)
            }
            None => Err(std::io::ErrorKind::AddrNotAvailable.into()),
        }
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
        use MapDir::*;
        let loc = self.loc_of(u.uuid())?;

        let u = u.uuid();
        Some(match loc.move_player(self, u, dir.clone()) {
            Ok(_) => format!("you go {}{}", dir, self.describe_room(u)?),
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

    fn get_player_mut<U: Uuid>(&mut self, u: U) -> Option<&mut Player> {
        self.players.get_mut(&u.uuid())
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
        let loc = &self.loc_of(u.uuid())?;
        let uuid = &u.uuid();

        let rooms = &mut self.rooms;
        let mut players = &mut self.players;
        match dir {
            Take => {
                rooms.get_mut(loc)?.transfer(players.get_mut(uuid)?, handle);
            }
            Drop => {
                players.get_mut(uuid)?.transfer(rooms.get_mut(loc)?, handle);
            }
            Give => {
                let item = players.get_mut(uuid)?.remove_item(handle)?;
                loc.player_by_name_mut(self, other?)?.give_item(item);
            }
            Wear => {
                let (mut items, mut clothing) = players.get_mut(uuid)?.all_items_mut();
                items.transfer(clothing, handle)?;
            }
            Remove => {
                let (mut items, mut clothing) = players.get_mut(uuid)?.all_items_mut();
                clothing.transfer(items, handle)?;
            }
        }

        Ok(())
    }
}

fn article(noun: &str) -> String {
    let suffix = match noun.to_lowercase().chars().next().unwrap_or('\0') {
        'a' | 'e' | 'i' | 'o' | 'u' => "n",
        _ => "",
    };

    format!("a{} {}", suffix, noun)
}

fn fill_interpreter(i: &mut Interpreter) {
    i.insert("look", |g, u, args| {
        Some(match args.len() {
            0 => g.describe_room(u)?,
            1 => {
                if let Some(item) = g.describe_item(u, args[0]) {
                    item.to_owned()
                } else if let Some(person) = g.describe_player(u, args[0]) {
                    person.to_owned()
                } else {
                    format!("i don't see {} here...", article(args[0]))
                }
            }
            _ => "be more specific. or less specific.".to_owned(),
        })
    });

    i.insert("take", |g, u, a| match a.len() {
        0 => Some("there seems to be an error".to_owned()),
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Take, handle) {
                    format!("you take the {}", Red(handle.to_owned()))
                } else {
                    format!("you don't see {} here", Green(article(handle)))
                },
            )
        }
        _ => Some("be more specific. or less specific.".to_owned()),
    });

    i.insert("wear", |g, u, a| match a.len() {
        0 => Some("there seems to be an error".to_owned()),
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Wear, handle) {
                    format!("you wear the {}", handle)
                } else {
                    format!("you're not holding {}", article(handle))
                },
            )
        }
        _ => Some("be more specific. or less specific.".to_owned()),
    });

    i.insert("remove", |g, u, a| match a.len() {
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Remove, handle) {
                    format!("you take off the {}", handle)
                } else {
                    format!("you're not wearing {}", article(handle))
                },
            )
        }
        _ => Some("be more specific. or less specific.".to_owned()),
    });

    i.insert("drop", |g, u, a| match a.len() {
        0 => Some("there seems to be an error".to_owned()),
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Drop, handle) {
                    format!("you drop the {}", handle)
                } else {
                    format!("you don't see {} here", article(handle))
                },
            )
        }
        _ => Some("be more specific. or less specific.".to_owned()),
    });

    i.insert("give", |g, u, a| match a.len() {
        2 => {
            let (other, handle) = (a[0], a[1]);
            Some(
                if g.transfer(u, Some(other), Direction::Give, handle).is_ok() {
                    format!("you give {} {}", other, article(handle))
                } else {
                    "that person or thing isn't here".to_owned()
                },
            )
        }
        _ => Some("E - NUN - CI - ATE".to_owned()),
    });

    i.insert("say", |g, u, a| {
        let message = a.join(" ");
        let loc = g.loc_of(u)?;
        let name = g.name_of(u)?.to_owned();
        let players: Vec<u128> = loc.player_ids(g)?.iter().cloned().collect();

        for p in players {
            if p == u {
                continue;
            }

            g.send_to_player(p, format!("\n{} says '{}'\n", name, message))
                .ok()?;
        }

        Some(format!("you say '{}'", message))
    });

    i.insert("chat", |g, u, a| {
        let statement = a.join(" ");
        let name = g.name_of(u)?.to_owned();

        let all_players: Vec<u128> = g.players.keys().filter(|id| **id != u).cloned().collect();
        for p in all_players {
            g.send_to_player(p, format!("{} chats '{}'", name, statement))
                .ok()?;
        }

        Some(format!("you chat '{}'", statement))
    });

    i.insert("evaluate", |g, u, _| {
        let p = g.get_player(u)?;

        let mut s = String::new();
        for meter in p.stats() {
            s.push_str(&format!("{:#?}", meter));
        }

        Some(s)
    });

    i.insert("north", |g, u, _| g.dir_func(u, MapDir::North));

    i.insert("south", |g, u, _| g.dir_func(u, MapDir::South));

    i.insert("ouch", |g, u, a| {
        const prick: usize = 5;
        g.players.entry(u).or_default().hurt(prick);

        Some(format!("that hurt a surprising amount"))
    });

    i.insert("inventory", |g, u, _a| g.list_inventory(u));

    i.insert("", |_, _, _| Some("".to_owned()));

    i.insert("none", |_, _, _| Some(random_insult()));

    i.insert("quit", |_, _, _| return None)
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

#[cfg(test)]
mod game_test {
    use super::*;
    use std::borrow::{Borrow, BorrowMut};
    use std::sync::RwLock;

    struct Thing {
        inner: HashMap<usize, RwLock<Room>>,
    }

    impl Thing {
        fn new() -> Self {
            let mut inner = HashMap::new();
            let mut g = Game::new();
            let r1 = take(g.rooms.get_mut(&Coord(0, 0)).unwrap());
            let r2 = take(g.rooms.get_mut(&Coord(0, 1)).unwrap());
            inner.insert(0, RwLock::new(r1));
            inner.insert(1, RwLock::new(r2));

            Thing { inner }
        }

        // DOES NOT WORK
        fn thing(&mut self) -> Option<String> {
            let r2 = self.inner.get_mut(&0).unwrap().read().unwrap().borrow();
            let mut r1 = self.inner.get(&1).unwrap().write().unwrap().borrow_mut();
            r1.add_player(&(7 as u128));
            let x = r2.get_item("codpiece").unwrap();
            Some(format!("{} {}", r1.players().len(), x.name()))
        }
    }

    #[test]
    fn test_interior_mutability() {
        assert_eq!(Thing::new().thing(), Some("1 codpiece".to_owned()));
    }

    fn new_game() -> Game {
        let mut g = Game::new();
        let p = Player::new("peter");

        let uuid = p.uuid();
        g.add_player(p);
        g
    }

    #[test]
    fn game_test_display_room() {
        let p = Player::new("lol");
        let uuid = p.uuid();
        let q = Player::new("billy");
        let pp = Player::new("mindy");

        let mut r = Room::new("the room", None);
        let mut g = Game::new();
        for player in vec![p, q, pp] {
            r.add_player(&player);
            g.players.insert(player.uuid(), player);
        }
        g.rooms.insert(Coord(0, 0), r);

        println!("{}", g.describe_room(uuid)?);
    }

    #[test]
    fn game_test_interpreter() {
        let mut g = Game::new();
        let mut r = Room::new("yo", None);
        let p = Player::new("lol");
        r.add_player(&p);
        let uuid = p.uuid();
        g.add_player(p);

        assert!(g.interpret(uuid, "look").is_some());
    }
}
