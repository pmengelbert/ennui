use ennui::*;
use std::collections::HashMap;
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

    let mut c = interpreter![
        look, 
        say,
        status,
        take,
        inventory,
        drop,
        quit,
        wear,
        remove,
        north
    ];

    let mut m = Map {
        map: HashMap::new()
    };

    let mut room = Room::new("kitchen".to_string(), "this is the kitchen".to_string());

    let item = Item {
        kind: ItemType::Normal,
        name: "a book".to_string(),
        description: "a nice book".to_string()
    };
    room.items.insert("book".to_string(), item);

    let item = Item {
        kind: ItemType::Wearable,
        name: "a shirt".to_string(),
        description: "a nice book".to_string()
    };
    room.items.insert("shirt".to_string(), item);

    let mut other = Room::new("other".to_string(), "this is the other".to_string());

    m.map.insert(Coord(0, 0), room);
    m.map.insert(Coord(0, 1), other);
    let r = m.map.get_mut(&Coord(0, 0)).unwrap();
    let mut bill = Player::new("bill", &mut m);

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
