use std::collections::HashMap;
use std::mem::{swap, take};
use std::option::NoneError;
use std::process;

use crate::interpreter::Interpreter;
use crate::item::{Item, ItemKind, ItemList};
use crate::map::{Coord, Room};
use crate::player::{Player, PlayerList, Uuid};

use rand::Rng;

type PassFail = Result<(), std::option::NoneError>;

pub struct Game {
    players: PlayerList,
    rooms: HashMap<Coord, Room>,
    interpreter: Interpreter,
}

enum Direction {
    Take,
    Give,
    Drop,
}

macro_rules! cleanup_on_fail {
    ($res:expr, $label:tt) => {
        match $res {
            Some(r) => r,
            None => break $label,
        }
    };
}

macro_rules! with_cleanup {
    (($label:tt) $code:expr) => {
        $label: loop {
            $code

            break $label
        }
    }
}

impl Game {
    pub fn new() -> Self {
        let (players, mut rooms) = (HashMap::new(), HashMap::new());
        let mut r = Room::new("the living room", Some("this is the living room"));
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

    fn display_room(&self, c: &Coord) -> String {
        match self.rooms.get(c) {
            Some(r) => r.display(&self.players),
            None => "".to_owned(),
        }
    }

    pub fn interpret(&mut self, p: u128, s: &str) -> Option<String> {
        let mut interpreter = take(&mut self.interpreter);

        let ret = interpreter.interpret(self, p, s);

        self.interpreter = interpreter;
        ret
    }

    pub fn add_player(&mut self, p: Player) {
        self.players.insert(p.uuid(), p);
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

                format!("{}\n{}", p.description().to_owned(), item_list)
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
            let p = cleanup_on_fail!(players.get_mut(&u.uuid()), 'outer_cleanup);
            let mut p = take(p);

            let r = cleanup_on_fail!(rooms.get_mut(p.loc()), 'outer_cleanup);

            let mut players_items = p.get_itemlist();
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
                        let other = cleanup_on_fail!(other, 'inner_cleanup);
                        let other_player = cleanup_on_fail!(r.players().get_player_mut_by_name(&mut players, other), 'inner_cleanup);

                        let mut others_items = other_player.get_itemlist();
                        let inner_result = Self::t_item(&mut players_items, &mut others_items, handle);
                        other_player.replace_itemlist(others_items);

                        inner_result
                    }
                };
            });

            // 'inner_cleanup:
            r.replace_itemlist(room_items);
            p.replace_itemlist(players_items);

            let q = players.entry(u.uuid()).or_default();
            swap(q, &mut p);
        });

        // 'outer_cleanup:
        self.rooms = rooms;
        self.players = players;

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
}

fn article(noun: &str) -> String {
    let suffix = if let Some(c) = noun.to_lowercase().chars().next() {
        match c {
            'a' | 'e' | 'i' | 'o' | 'u' => "n",
            _ => "",
        }
    } else {
        ""
    };

    format!("a{} {}", suffix, noun)
}

fn fill_interpreter(i: &mut Interpreter) {
    i.insert("look", |g, u, args| {
        let player = g.get_player(u)?;
        let c = player.loc();
        match args.len() {
            0 => Some(g.display_room(c)),
            1 => {
                if let Some(item) = g.describe_item(u, args[0]) {
                    Some(item.to_owned())
                } else if let Some(person) = g.describe_player(u, args[0]) {
                    Some(person.to_owned())
                } else {
                    Some(format!("i don't see {} here...", article(args[0])))
                }
            }
            _ => None,
        }
    });

    i.insert("take", |g, u, a| match a.len() {
        0 => Some("there seems to be an error".to_owned()),
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Take, handle) {
                    format!("you take the {}", handle)
                } else {
                    format!("you don't see {} here", article(handle))
                },
            )
        }
        _ => None,
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
        _ => None,
    });

    i.insert("give", |g, u, a| match a.len() {
        2 => {
            if let &[other, handle, ..] = a {
                if g.transfer(u, Some(other), Direction::Give, handle).is_ok() {
                    Some(format!("you give {} {}", other, article(handle)))
                } else {
                    Some("that person or thing isn't here".to_owned())
                }
            } else {
                None
            }
        }
        _ => Some("E - NUN - CI - ATE".to_owned()),
    });

    i.insert("inventory", |g, u, _a| g.list_inventory(u));

    i.insert("none", |_, _, _| Some(random_insult()));

    i.insert("quit", |_, _, _| {
        process::exit(0);
    })
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
        let q = Player::new("billy");
        let pp = Player::new("mindy");

        let mut r = Room::new("the room", None);
        let mut g = Game::new();
        for player in vec![p, q, pp] {
            r.add_player(&player);
            g.players.insert(player.uuid(), player);
        }
        g.rooms.insert(Coord(0, 0), r);

        println!("{}", g.display_room(&Coord(0, 0)));
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
