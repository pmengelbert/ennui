use ennui::*;
mod game;
use std::io::{self, Write};

macro_rules! interpreter {
    ( $( $name:ident ),* ) => {
        {
            let mut i = Interpreter::new();
            $( i.set(stringify!($name), $name); )*
            i
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut bill = Player::new("bill");

    let g = game::Game::new();

    Ok(())
}
