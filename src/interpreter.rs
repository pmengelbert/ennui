use std::collections::HashMap;

use crate::error::EnnuiError;
use crate::game::Game;
use crate::interpreter::CommandQuality::{Awake, Motion};
use crate::item::Attribute;
use crate::text::message::{Message, Messenger};
use std::sync::{Arc, Mutex};

pub type CommandMessage = (Box<dyn Messenger>, Box<dyn Message>);
pub type CommandFunc = Arc<
    Mutex<dyn FnMut(&mut Game, u128, &[&str]) -> Result<CommandMessage, EnnuiError> + Send + Sync>,
>;

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
    Hit,
    Sleep,
    Stand,
    Wake,
    Quit,
    // not yet implemented
    #[allow(dead_code)]
    Whisper,
    #[allow(dead_code)]
    Blank,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CommandQuality {
    Awake,
    Motion,
}

const NORTH_QUALITIES: [CommandQuality; 2] = [Awake, Motion];
const BLANK_QUALITIES: [CommandQuality; 0] = [];

impl Attribute<CommandQuality> for CommandKind {
    fn attr(&self) -> Vec<CommandQuality> {
        use CommandKind::*;

        match self {
            North => &NORTH_QUALITIES[..],
            _ => &BLANK_QUALITIES[..],
        }
        .to_vec()
    }

    fn set_attr(&mut self, _: CommandQuality) {
        unimplemented!()
    }

    fn unset_attr(&mut self, _: CommandQuality) {
        unimplemented!()
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
            s if sw(s, "sleep") => Sleep,
            s if sw(s, "stand") => Stand,
            s if sw(s, "wake") => Wake,
            s if sw(s, "hit") => Hit,
            s if sw(s, "quit") => Quit,
            _ => NotFound,
        }
    }

    pub fn insert<F: 'static>(&mut self, c: &str, f: F)
    where
        F: FnMut(
                &mut Game,
                u128,
                &[&str],
            ) -> Result<(Box<dyn Messenger>, Box<dyn Message>), EnnuiError>
            + Send
            + Sync,
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
    F: FnMut(&mut Game, u128, &[&str]) -> Result<CommandMessage, EnnuiError> + Send + Sync,
{
    Arc::new(Mutex::new(cf))
}
