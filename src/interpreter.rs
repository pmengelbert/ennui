use std::collections::HashMap;
use std::ops::Deref;

use crate::game::Game;
use std::sync::{Mutex, Arc};

#[derive(Eq, PartialEq, Debug, Hash)]
pub enum CommandKind {
    Look,
    Take,
    Drop,
    Give,
    Wear,
    Remove,
    Inventory,
    NotFound,
    Quit,
}

type CommandFunction = Arc<Mutex<dyn FnMut(&mut Game, u128, &[&str]) -> Option<String>>>;
struct CommandFunc(CommandFunction);
impl Deref for CommandFunc {
    type Target = CommandFunction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Send for CommandFunc {}

impl Default for CommandFunc {
    fn default() -> Self {
        b(|_, _, _| None)
    }
}

#[derive(Default)]
pub struct Interpreter {
    commands: HashMap<CommandKind, CommandFunc>,
}

impl Interpreter {
    pub fn new() -> Self {
        let commands: HashMap<CommandKind, CommandFunc> = HashMap::new();
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
            s if sw(s, "look") => Look,
            s if sw(s, "take") => Take,
            s if sw(s, "drop") => Drop,
            s if sw(s, "give") => Give,
            s if sw(s, "wear") => Wear,
            s if sw(s, "remove") => Remove,
            s if sw(s, "inventory") => Inventory,
            s if sw(s, "quit") => Quit,
            _ => NotFound,
        }
    }

    pub fn insert<F: 'static>(&mut self, c: &str, f: F)
    where
        F: FnMut(&mut Game, u128, &[&str]) -> Option<String>,
    {
        self.commands.insert(Self::resolve_str(c), b(f));
    }

    pub fn interpret(&mut self, g: &mut Game, pid: u128, s: &str) -> Option<String> {
        let mut tokens = s.split_whitespace();
        let (cmd, args) = (tokens.next().unwrap_or(""), tokens.collect::<Vec<_>>());
        let cmd = Self::resolve_str(cmd);
        let CommandFunc(cmd_func) = self.commands.entry(cmd).or_default();

        cmd_func.lock().unwrap()(g, pid, &args)
    }
}

fn b<F: 'static>(cf: F) -> CommandFunc
where
    F: FnMut(&mut Game, u128, &[&str]) -> Option<String>,
{
    CommandFunc(Arc::new(Mutex::new(cf)))
}

#[cfg(test)]
mod interpreter_test {
    use super::*;

    #[test]
    fn interpreter_new_test() {
        use CommandKind::*;
        let mut i = Interpreter::new();
        i.commands
            .insert(Look, b(|g, pid, args| Some("you have looked".to_owned())));
    }

    #[test]
    fn interpreter_shortened_commands_resolve_correctly() {
        assert_eq!(Interpreter::resolve_str("look"), CommandKind::Look);
        assert_eq!(Interpreter::resolve_str("loo"), CommandKind::Look);
        assert_eq!(Interpreter::resolve_str("lo"), CommandKind::Look);
        assert_eq!(Interpreter::resolve_str("l"), CommandKind::Look);
    }

    #[test]
    fn interpreter_resolve_String_works_too() {
        assert_eq!(
            Interpreter::resolve_str(String::from("look")),
            CommandKind::Look
        );
    }

    #[test]
    fn interpreter_resolves_case_insensitively() {
        assert_eq!(Interpreter::resolve_str("tA"), CommandKind::Take);
    }
}
