use ennui::game::Game;
use ennui::interpreter::commands;
#[macro_use]
use ennui::interpreter;
use ennui::interpreter::{random_insult, Interpreter};
use ennui::map::Coord;
use ennui::player::{Player, PlayerType::*};
use std::io;
use std::io::Write;

fn main() -> Result<(), String> {
    let p = Player::new("bill");
    let mut dog = Player::new("dog");
    dog.set_description("an adorable pup");
    let dog_uuid = dog.uuid();

    let uuid = p.uuid();

    let mut g = Game::new();

    let c = interpreter![
        commands::look,
        north,
        south,
        east,
        west,
        take,
        loc,
        drop,
        quit,
        inventory,
        remove,
        wear,
        say
    ];

    g.add_player(Human(p));
    g.add_player(NPC(dog));
    g.place_player_in_room(dog_uuid, Coord(0, 0))?;
    g.place_player_in_room(uuid, Coord(0, 0))?;

    println!("{}", g.room_to_string_for_player(uuid));

    loop {
        let mut user_input = String::new();
        print!("\n> ");
        io::stdout().flush();

        io::stdin()
            .read_line(&mut user_input)
            .expect("failed to read input");

        let s = user_input.trim_end();
        let x = s.split_whitespace().collect::<Vec<&str>>();

        if let Some(cf) = c.get(&x[0]) {
            println!("{}", cf(&mut g, uuid, &x[1..]));
        } else {
            println!("{}", random_insult());
        }
    }

    Ok(())
}
