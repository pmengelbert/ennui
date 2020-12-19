use std::collections::HashMap;

pub enum Command {
    Look,
}

pub struct Interpreter {
    commands: HashMap<Command, Box<dyn Fn(usize) -> usize>>,
}