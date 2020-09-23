use super::game::Game;
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
}
