use crate::hook::Hook;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait Describe: Send + Sync + Debug {
    fn name(&self) -> String;
    fn display(&self) -> String;
    fn description(&self) -> String;
    fn handle(&self) -> Hook;
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Description {
    pub name: String,
    pub display: String,
    pub description: String,
    pub handle: Hook,
}

impl Describe for Description {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn display(&self) -> String {
        self.display.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn handle(&self) -> Hook {
        self.handle.clone()
    }
}

impl Description {
    pub fn new(name: &str, description: Option<&str>, handle: Hook) -> Self {
        let description = description.unwrap_or_default().to_owned();
        let name = name.to_owned();
        let display = String::new();

        Self {
            name,
            description,
            handle,
            display,
        }
    }
}

#[macro_export]
macro_rules! handle {
    ( $( $name:ident ),* ) => {
        {
            #[allow(unused_mut)]
            let mut h: crate::hook::Hook = crate::hook::Hook(Vec::new());
            $( h.push(stringify!($name).to_owned()); )*
            h
        }
    }
}

#[cfg(test)]
mod handle_test {
    use super::*;

    #[test]
    fn handle_test() {
        let h = handle![sword, rusty];
        eprintln!("{:#?}", h);
        eprintln!("in file {} on line number {}", file!(), line!());

        assert_eq!("sword", h);
        assert_eq!("rusty", h);
    }
}
