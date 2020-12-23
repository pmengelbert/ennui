use std::collections::HashMap;
use std::mem::{swap, take};
use std::option::NoneError;
use std::{io, process};

use crate::interpreter::Interpreter;
use crate::item::{Item, ItemKind, ItemList};
use crate::map::{Coord, Room};
use crate::player::{Player, PlayerList, Uuid};
use crate::text::{Color::*, Wrap};

use rand::Rng;
use std::cmp::min;
use std::io::Write;

type PassFail = Result<(), std::option::NoneError>;

pub struct Game {
    players: PlayerList,
    rooms: HashMap<Coord, Room>,
    interpreter: Interpreter,
}

pub trait Transfer {
    fn transfer() -> Option<()>;
}

// impl Send for Game {}

enum Direction {
    Take,
    Give,
    Drop,
    Wear,
    Remove,
}

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

impl Game {
    pub fn new() -> Self {
        let (players, mut rooms) = (HashMap::new(), HashMap::new());
        let desc = r#"You are at the Temple Yard of Dragonia. Beautiful marble stairs lead up to the Temple of Dragonia. You feel small as you stare up the huge pillars making the entrance to the temple. This place serves as a sanctuary where the people of the city can come and seek refuge, and rest their tired bones. Just north of here is the common square, and the temple opens to the south."#;
        let mut r = Room::new("the living room", Some(desc));
        let mut r2 = Room::new(
            "the other room",
            Some(&desc.chars().rev().collect::<String>()),
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
        rooms.insert(Coord(0, 0), r);
        rooms.insert(Coord(0, 1), r2);
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

    fn display_room<P: Uuid>(&mut self, p: P) -> String {
        let mut player = take(self.players.entry(p.uuid()).or_default());
        let mut ret = "".to_owned();

        with_cleanup!(('player_cleanup) {
            let c = player.loc();
            let r = goto_cleanup_on_fail!(self.rooms.get(c), 'player_cleanup);

            ret = r.display(p.uuid(), &self.players);
        } 'cleanup: {
            swap(self.players.entry(p.uuid()).or_default(), &mut player);
        });

        ret
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
        self.rooms.entry(*p.loc()).or_default().add_player(&p);
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

    fn move_player<U: Uuid>(&self, u: U, from: &mut Room, to: &mut Room) -> Option<()> {
        let u = &u.uuid();
        from.players_mut().remove(u);
        to.add_player(u);
        Some(())
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

        let loc = p.loc();
        let room = self.rooms.get(loc)?;

        Some(if let Some(item) = room.get_item(handle) {
            &item.description()
        } else {
            p.items().get(handle)?.description()
        })
    }

    fn dir_func<U: Uuid>(&mut self, u: U, dir: MapDir) -> Option<String> {
        use MapDir::*;
        let u = u.uuid();
        let loc = self.loc_of(u)?;
        let other_loc = loc.add(dir);

        let mut fail = true;
        let mut ret = None;
        let mut rooms = take(&mut self.rooms);

        with_cleanup!(('rooms) {
            let mut next_room = take(goto_cleanup_on_fail!(rooms.get_mut(&other_loc), 'rooms));
            let mut current_room = take(goto_cleanup_on_fail!(rooms.get_mut(&loc), 'rooms));
            with_cleanup!(('inner) {
                goto_cleanup_on_fail!(self.move_player(u, &mut current_room, &mut next_room), 'inner);
                goto_cleanup_on_fail!(self.players.get_mut(&u), 'inner).set_loc(other_loc);
                fail = false;

            } 'cleanup: {
                swap(rooms.entry(other_loc).or_default(), &mut next_room);
                swap(rooms.entry(loc).or_default(), &mut current_room);
            });

        } 'cleanup: {
            self.rooms = rooms;
            ret = Some(if fail {
                "alas! you cannot go that way".to_owned()
            } else {
                self.display_room(u)
            })
        });

        ret
    }

    fn describe_player<T>(&self, pid: T, other: &str) -> Option<String>
    where
        T: Uuid,
    {
        let room = {
            let p = self.get_player(pid.uuid())?;

            let loc = p.loc();
            self.rooms.get(loc)?
        };

        Some(
            if let Some(p) = room.players().get_player_by_name(&self.players, other) {
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

                format!("{}{}", p.description().to_owned(), item_list)
            } else {
                format!("you don't see {} here", other)
            },
        )
    }

    fn transfer<T>(&mut self, u: T, other: Option<&str>, dir: Direction, handle: &str) -> PassFail
    where
        T: Uuid,
    {
        use Direction::*;

        let mut rooms = take(&mut self.rooms);
        let mut players = take(&mut self.players);

        let mut ret = Err(NoneError);

        with_cleanup!(('outer_cleanup) {
            let p = goto_cleanup_on_fail!(players.get_mut(&u.uuid()), 'outer_cleanup);
            let mut p = take(p);

            let r = goto_cleanup_on_fail!(rooms.get_mut(p.loc()), 'outer_cleanup);

            let mut players_items = p.get_itemlist();
            let mut players_clothing = p.get_clothinglist();
            let mut room_items = r.get_itemlist();

            with_cleanup!(('inner_cleanup) {
                ret = match dir {
                    Take => {
                        Self::t_item(&mut room_items, &mut players_items, handle)
                    }
                    Drop => {
                        Self::t_item(&mut players_items, &mut room_items, handle)
                    }
                    Give => {
                        let other = goto_cleanup_on_fail!(other, 'inner_cleanup);
                        let other_player = goto_cleanup_on_fail!(r.players().get_player_mut_by_name(&mut players, other), 'inner_cleanup);

                        let mut others_items = other_player.get_itemlist();
                        let inner_result = Self::t_item(&mut players_items, &mut others_items, handle);
                        other_player.replace_itemlist(others_items);

                        inner_result
                    }
                    Wear => {
                        Self::t_item(&mut players_items, &mut players_clothing, handle)
                    }
                    Remove => {
                        Self::t_item(&mut players_clothing, &mut players_items, handle)
                    }
                };
            } 'cleanup: {
                // 'inner_cleanup:
                r.replace_itemlist(room_items);
                p.replace_itemlist(players_items);
                p.replace_clothinglist(players_clothing);

                let q = players.entry(u.uuid()).or_default();
                swap(q, &mut p);
            });

        } 'cleanup: {
            // 'outer_cleanup:
            self.rooms = rooms;
            self.players = players;
        });

        ret
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

    fn t_item(from: &mut ItemList, to: &mut ItemList, handle: &str) -> PassFail {
        let item = from.get_owned(handle)?;
        to.push(item);
        Ok(())
    }

    fn loc_of<P>(&self, p: P) -> Option<Coord>
    where
        P: Uuid,
    {
        Some(*self.players.get(&p.uuid())?.loc())
    }

    #[allow(dead_code)]
    fn name_of<P>(&self, p: P) -> Option<&str>
    where
        P: Uuid,
    {
        Some(self.players.get(&p.uuid())?.name())
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
            0 => g.display_room(u),
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
        let statement = a.join(" ");
        let loc = g.loc_of(u)?;
        let name = g.players.get(&u)?.name().to_owned();

        let mut ret = Some(format!("there's a pretty serious error here"));

        let rooms = take(&mut g.rooms);
        with_cleanup!(('rooms_cleanup) {
            let s = format!("{} says '{}'", name, statement);
            let r = goto_cleanup_on_fail!(rooms.get(&loc), 'rooms_cleanup).players();

            for id in &**r {
                if *id == u {
                    continue;
                }

                g.send_to_player(*id, s.clone()).ok()?;
            }
            ret = Some(format!("you say '{}'", statement));
        } 'cleanup: {
            g.rooms = rooms;
        });

        ret
    });

    i.insert("chat", |g, u, a| {
        let statement = a.join(" ");
        let mut ret = Some(format!("there's a pretty serious error here"));

        let mut p = take(g.players.entry(u).or_default());
        with_cleanup!(('player_cleanup) {
            goto_cleanup_on_fail!(p.broadcast(&mut g.players, statement.clone()).ok(), 'player_cleanup);
            ret = Some(format!("you chat '{}'", statement));
        } 'cleanup: {
            swap(g.players.entry(u).or_default(), &mut p);
        });

        ret
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

    // i.insert("ouch", |g, u, a| {
    //     const prick: usize = 5;
    //     g.players.entry(u).or_default().hurt(prick);

    //     Some(format!("that hurt a surprising amount"))
    // });

    i.insert("inventory", |g, u, _a| g.list_inventory(u));

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

        println!("{}", g.display_room(uuid));
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
