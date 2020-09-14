use std::collections::HashMap;

pub struct Interpreter<F> {
    commands: HashMap<String, F>,
}

impl<F> Interpreter<F> 
where F: Fn(&str, &str)
{
    pub fn new() -> Self {
        Interpreter {
            commands: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: F) -> Option<F> {
        self.commands.insert(key, value)
    }

    pub fn get(&mut self, key: &str) -> Option<&F> {
        self.commands.get(key)
    }
}
