use serde::export::Formatter;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Clothing(String),
    ItemNotFound(String),
    PlayerNotFound(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {:#?}", self)
    }
}

impl std::error::Error for Error {}
