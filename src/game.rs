use super::player::Player;
use std::collections::HashMap;
use crate::map::{Coord, Room};
use crate::player::{PlayerListRaw, PlayerList, Uuid};
use crate::interpreter::Interpreter;
use std::process;
use crate::item::{ItemKind, Item};

pub struct Game {
    players: HashMap<u128, Player>,
    rooms: HashMap<Coord, Room>,
    interpreter: Interpreter,
}

impl Game {
    pub fn new() -> Self {
        let (players, mut rooms) = (HashMap::new(), HashMap::new());
        let mut r = Room::new("the living room", Some("this is the living room"));
        let p = Player::new("billy");
        r.add_player(&p);
        let i = ItemKind::Clothing(Item::new("codpiece", Some("a beautifully decorated codpiece. truly a wonder"), "codpiece"));
        r.add_item(i);
        rooms.insert(Coord(0, 0), r);
        let mut interpreter = Interpreter::new();
        fill_interpreter(&mut interpreter);
        let mut ret = Self {
            players,
            rooms,
            interpreter,
        };

        ret.add_player(p);
        ret
    }

    pub fn display_room(&self, c: &Coord) -> String {
        match self.rooms.get(c) {
            Some(r) => {
                r.display(&self.players)
            },
            None => "".to_owned(),
        }
    }

    pub fn players(&self) -> &HashMap<u128, Player> {
        &self.players
    }

    pub fn interpret(&mut self, p: u128, s: &str) -> Option<String> {
        let i = &mut self.interpreter;
        let mut interpreter = std::mem::replace(i, Interpreter::new());

        let ret = interpreter.interpret(self, p, s);
        self.interpreter = interpreter;
        ret
    }

    pub fn add_player(&mut self, p: Player) {
        self.players.insert(p.uuid(), p);
    }

    pub fn get_player(&self, u: u128) -> Option<&Player> {
        self.players.get(&u)
    }

    pub fn describe_item<U>(&self, pid: U, handle: &str) -> Option<&str>
        where U: Uuid,
    {
        let p = self.get_player(pid.uuid())?;

        let loc = p.loc();
        let items = self.rooms.get(loc)?;

        if let Some(item) = items.get_item(handle) {
            Some(&item.description())
        } else {
            Some(p.items().get(handle)?.description())
        }
    }
}

fn fill_interpreter(i: &mut Interpreter) {
    i.insert("look", |g, u, args| {
        let player = g.get_player(u).or(None)?;
        let c = player.loc();
        match args.len() {
            0 => {
                Some(g.display_room(c))
            },
            1 => {
                Some(g.describe_item(u, args[0])?.to_owned())
            }
            _ => None,
        }
    });
    i.insert("quit", |g, u, a| {
        process::exit(0);
    })
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