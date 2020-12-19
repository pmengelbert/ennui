use super::player::Player;
use std::collections::HashMap;
use crate::map::{Coord, Room};
use crate::player::{PlayerListRaw, PlayerList};
use crate::interpreter::Interpreter;

pub struct Game {
    players: HashMap<u128, Player>,
    rooms: HashMap<Coord, Room>,
    interpreter: Interpreter,
}

impl Game {
    pub fn new() -> Self {
        let (players, mut rooms) = (HashMap::new(), HashMap::new());
        rooms.insert(Coord(0, 0), Room::new("the living room", Some("this is the living room")));
        let mut interpreter = Interpreter::new();
        fill_interpreter(&mut interpreter);
        Self {
            players,
            rooms,
            interpreter,
        }
    }

    pub fn display_room(&self, c: &Coord) -> String {
        match self.rooms.get(c) {
            Some(r) => r.display(&self.players),
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
}

fn fill_interpreter(i: &mut Interpreter) {
    i.insert("look", |g, u, args| {
        match args.len() {
            0 => {
                let c = g.get_player(u).unwrap().loc();
                Some(g.display_room(c))
            },
            _ => None,
        }
    });
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

        let p = Player::new("lol");
        println!("{}", g.interpret(p.uuid(), "look there").unwrap());
    }
}