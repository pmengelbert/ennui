use std::collections::HashMap;
use std::ops::Deref;

use crate::game::Game;
use std::sync::{Arc, Mutex};

type CommandFunction = Arc<Mutex<dyn FnMut(&mut Game, u128, &[&str]) -> Option<String>>>;
pub struct CommandFunc(CommandFunction);

#[derive(Default)]
pub struct Interpreter {
    commands: Arc<Mutex<HashMap<CommandKind, CommandFunc>>>,
}

#[derive(Eq, PartialEq, Debug, Hash)]
pub enum CommandKind {
    North,
    South,
    East,
    West,
    Look,
    Take,
    Drop,
    Give,
    Wear,
    Remove,
    Chat,
    Say,
    Eval,
    Inventory,
    NotFound,
    Ouch,
    Open,
    Unlock,
    Quit,
    // not yet implemented
    #[allow(dead_code)]
    Whisper,
    #[allow(dead_code)]
    Blank,
}

impl Deref for CommandFunc {
    type Target = CommandFunction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Send for CommandFunc {}

impl Default for CommandFunc {
    fn default() -> Self {
        b(|_, _, _| Some("".into()))
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let commands = arc_mutex!(HashMap::new());
        Self { commands }
    }

    pub fn resolve_str<T>(s: T) -> CommandKind
    where
        T: AsRef<str>,
    {
        use CommandKind::*;
        let sw = |s, str: &str| str.starts_with(s);

        let s = s.as_ref();

        match s.to_lowercase().as_str() {
            s if s.is_empty() => NotFound,
            s if sw(s, "north") => North,
            s if sw(s, "south") => South,
            s if sw(s, "east") => East,
            s if sw(s, "west") => West,
            s if sw(s, "look") => Look,
            s if sw(s, "take") => Take,
            s if sw(s, "get") => Take,
            s if sw(s, "drop") => Drop,
            s if sw(s, "give") => Give,
            s if sw(s, "wear") => Wear,
            s if sw(s, "chat") => Chat,
            s if sw(s, "say") => Say,
            s if sw(s, "open") => Open,
            s if sw(s, "unlock") => Unlock,
            s if sw(s, "remove") => Remove,
            s if sw(s, "inventory") => Inventory,
            s if sw(s, "evaluate") => Eval,
            s if sw(s, "ouch") => Ouch,
            s if sw(s, "quit") => Quit,
            _ => NotFound,
        }
    }

    pub fn insert<F: 'static>(&mut self, c: &str, f: F)
    where
        F: FnMut(&mut Game, u128, &[&str]) -> Option<String>,
    {
        self.commands
            .lock()
            .unwrap()
            .insert(Self::resolve_str(c), b(f));
    }

    pub fn commands(&mut self) -> Arc<Mutex<HashMap<CommandKind, CommandFunc>>> {
        self.commands.clone()
    }

    pub fn process_string_command(s: &str) -> (CommandKind, Vec<&str>) {
        let mut words = s.split_whitespace();
        let cmd_str = words.next().unwrap_or_default();
        let args: Vec<&str> = words.collect();
        let cmd = Interpreter::resolve_str(cmd_str);
        (cmd, args)
    }
}

fn b<F: 'static>(cf: F) -> CommandFunc
where
    F: FnMut(&mut Game, u128, &[&str]) -> Option<String>,
{
    CommandFunc(Arc::new(Mutex::new(cf)))
}
