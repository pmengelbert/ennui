use super::game::Game;
use crate::map::Coord;
use crate::player::UUID;
use rand::Rng;
use std::collections::HashMap;

#[macro_export]
macro_rules! interpreter { ( $namespace:ident :: $( $name:ident ),* ) => {
        {
            let mut i = Interpreter::new();
            $( i.insert(stringify!($name), $namespace::$name); )*
            i
        }
    }
}

pub mod commands;

type CommandFunc = fn(&mut Game, UUID, &[&str]) -> String;

pub struct Interpreter {
    commands: HashMap<String, CommandFunc>,
}

pub fn random_insult() -> String {
    match rand::thread_rng().gen_range(1, 6) {
        1 => "dude wtf".to_string(),
        2 => "i think you should leave".to_string(),
        3 => "i'll have to ask my lawyer about that".to_string(),
        4 => "that's ... uncommon".to_string(),
        _ => "that's an interesting theory... but will it hold up in the laboratory?".to_string(),
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            commands: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: CommandFunc) -> Option<CommandFunc> {
        self.commands.insert(key.to_string(), value)
    }

    pub fn get(&self, key: &str) -> Option<&CommandFunc> {
        self.commands.get(key)
    }

    pub fn execute(g: &mut Game, uuid: UUID, command: &str, args: &[&str]) -> String {
        match command {
            "north" => {
                let Coord(x, y) = g.get_player(uuid).location();
                let new_coord = Coord(x, y + 1);

                match g.place_player_in_room(uuid, new_coord) {
                    Ok(msg) => format!("you go north\n{}", msg),
                    Err(s) => format!("you can't go that way!"),
                }
            }
            "loc" => {
                let Coord(x, y) = g.get_player(uuid).location();
                format!("you are standing at coordinate {},{}", x, y)
            }
            _ => format!("i'll have to ask my lawyer about that"),
        }
    }
}
