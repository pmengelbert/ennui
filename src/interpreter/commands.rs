use crate::game::{Direction, Game};
use crate::map::Coord;
use crate::player::UUID;

macro_rules! gen_func {
    ($name:ident ($g:ident, $uuid:ident, $args:ident) $bl:block ) => {
        #[allow(unused_variables)]
        pub fn $name($g: &mut Game, $uuid: UUID, $args: &[&str]) -> String {
            $bl
        }
    };
}

gen_func! {
    look(g, uuid, args) {
        match args.len() {
            0 => g.room_to_string_for_player(uuid),
            1 => g.look_at_item(uuid, args[0]),
            2 => match args[0] {
                "at" => {
                    g.look_at_item(uuid, args[1])
                },
                "in" => {
                    format!("not implemented yet")
                },
                _ => {
                    format!(r#""look at" or "look in", but don't "look {}""#, args[0])
                }
            }
            _ => format!("tell me ONE thing to look at, not a whole bunch at once"),
        }

    }
}

macro_rules! dir_func {
    ($name:ident:($x:expr, $y:expr)) => {
        gen_func! { $name(g, uuid, args) {
            let Coord(x, y) = g.get_player(uuid).location();
            let new_coord = Coord(x + $x, y + $y);

            match g.place_player_in_room(uuid, new_coord) {
                Ok(msg) => format!("you go {}\n{}", stringify!($name), msg),
                Err(_) => format!("you can't go that way!"),
            }
        }}
    };
}

dir_func! {north: ( 0,  1)}
dir_func! {south: ( 0, -1)}
dir_func! {east:  ( 1,  0)}
dir_func! {west:  (-1,  0)}

gen_func! {
    loc(g, uuid, args) {
        let Coord(x, y) = g.get_player(uuid).location();
        format!("you are standing at coordinate {},{}", x, y)
    }
}

gen_func! {
    take(g, uuid, args) {
        match args.len() {
            0 => format!("what do you want to take?"),
            1 => {
                match g.player_takes_item(uuid, args[0], Direction::From) {
                    Ok(msg) | Err(msg) => msg
                }
            },
            _ => {
                format!("whoa there big guy")
            }
        }
    }
}

gen_func! {
    drop(g, uuid, args) {
        match args.len() {
            0 => format!("what do you want to drop?"),
            1 => {
                match g.player_takes_item(uuid, args[0], Direction::To) {
                    Ok(msg) | Err(msg) => msg
                }
            },
            _ => {
                format!("whoa there big guy")
            }
        }
    }
}

gen_func! {
    inventory(g, uuid, args) {
        g.list_items_for_player(uuid)
    }
}

gen_func! {
    quit(g, uuid, args) {
        println!("goodbye");
        std::process::exit(0);
    }
}

gen_func! {
    say(g, uuid, args) {
        format!(r#"you say "{}""#, args.join(" "))
    }
}

gen_func! {
    wear(g, uuid, args) {
        match g.player_wears_item(uuid, args[0], Direction::To) {
            Ok(msg) | Err(msg) => msg,
        }
    }
}

gen_func! {
    remove(g, uuid, args) {
        match g.player_wears_item(uuid, args[0], Direction::From) {
            Ok(msg) | Err(msg) => msg,
        }
    }
}
