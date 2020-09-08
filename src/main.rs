use ennui::*;
mod game;
use std::io::{self, Write};

fn main() -> Result<(), std::io::Error> {
    let mut bill = Player::new("bill");

    let mut g = game::Game::new();
    let r = g.get_current_room(&bill).to_string();
    println!("{}", r);

    loop {
        let mut user_input = String::new();

        print!("\n> ");
        io::stdout().flush();

        io::stdin()
            .read_line(&mut user_input)?;

        let s = user_input.trim_end();

        let result = g.interpreter.execute_string(&mut bill, &s);
        println!("{}", result);
    }

    Ok(())
}
