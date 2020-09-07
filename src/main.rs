use ennui::*;
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

    let mut c = interpreter![
        look, 
        say,
        status,
        take,
        inventory,
        drop,
        quit,
        wear,
        remove
    ];

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
