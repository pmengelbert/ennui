use ennui::*;
use std::io::{self, Write};

fn main() -> Result<(), std::io::Error> {
    let mut bill = Player::new("bill");

    let mut c = Interpreter::new();
    c.set("look", look);
    c.set("say", say);
    c.set("status", status);
    c.set("take", take);
    c.set("inventory", inventory);
    c.set("drop", drop);
    c.set("quit", quit);
    c.set("wear", wear);
    c.set("remove", remove);
    c.set("rm", rm);

    loop {
        let mut user_input = String::new();

        print!("\n> ");
        io::stdout().flush();

        io::stdin()
            .read_line(&mut user_input)?;

        let s = user_input.trim_end();

        let result = c.execute_string(&mut bill, &s);
        println!("{}", result);
    }

    Ok(())
}
