use super::*;
use crate::text::message::{Audience, Msg};

pub fn fill_interpreter(i: &mut Interpreter) {
    i.insert("look", |g, u, args| {
        Some(match args.len() {
            0 => g.describe_room(u)?,
            1 => {
                if let Some(item) = g.describe_item(u, args[0]) {
                    item.to_owned()
                } else if let Some(person) = g.describe_player(u, args[0]) {
                    person.to_owned()
                } else {
                    format!("i don't see {} here...", article(args[0]))
                }
            }
            _ => "be more specific. or less specific.".to_owned(),
        })
    });

    i.insert("take", |g, u, a| match a.len() {
        0 => Some("there seems to be an error".to_owned()),
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Take, handle) {
                    format!("you take the {}", handle.to_owned())
                } else {
                    format!("you don't see {} here", article(handle))
                },
            )
        }
        _ => Some("be more specific. or less specific.".to_owned()),
    });

    i.insert("wear", |g, u, a| match a.len() {
        0 => Some("there seems to be an error".to_owned()),
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Wear, handle) {
                    format!("you wear the {}", handle)
                } else {
                    format!("you're not holding {}", article(handle))
                },
            )
        }
        _ => Some("be more specific. or less specific.".to_owned()),
    });

    i.insert("remove", |g, u, a| match a.len() {
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Remove, handle) {
                    format!("you take off the {}", handle)
                } else {
                    format!("you're not wearing {}", article(handle))
                },
            )
        }
        _ => Some("be more specific. or less specific.".to_owned()),
    });

    i.insert("drop", |g, u, a| match a.len() {
        0 => Some("there seems to be an error".to_owned()),
        1 => {
            let handle = a[0];
            Some(
                if let Ok(_) = g.transfer(u, None, Direction::Drop, handle) {
                    format!("you drop the {}", handle)
                } else {
                    format!("you don't see {} here", article(handle))
                },
            )
        }
        _ => Some("be more specific. or less specific.".to_owned()),
    });

    i.insert("give", |g, u, a| match a.len() {
        2 => {
            let (other, handle) = (a[0], a[1]);
            Some(
                if g.transfer(u, Some(other), Direction::Give, handle).is_ok() {
                    format!("you give {} {}", other, article(handle))
                } else {
                    "that person or thing isn't here".to_owned()
                },
            )
        }
        _ => Some("E - NUN - CI - ATE".to_owned()),
    });

    i.insert("say", |g, u, a| {
        let message = a.join(" ");
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let others = loc.player_ids(&g.rooms)?.clone();

        let aud = Audience(u, &others);
        let msg = Msg {
            s: format!("you say '{}'", message),
            o: Some(format!("{} says '{}'", name, message)),
        };

        let ret = Some(msg.s.to_owned());

        g.send(aud, msg);
        ret
    });

    i.insert("chat", |g, u, a| {
        let statement = a.join(" ");
        let name = g.name_of(u)?.to_owned();

        let all_players: Vec<u128> = g.players.keys().filter(|id| **id != u).cloned().collect();
        for p in all_players {
            g.send_to_player(p, format!("{} chats '{}'", name, statement))
                .ok()?;
        }

        Some(format!("you chat '{}'", statement))
    });

    i.insert("evaluate", |g, u, _| {
        let p = g.get_player(u)?;

        let mut s = String::new();
        for meter in p.stats() {
            s.push_str(&format!("{:#?}", meter));
        }

        Some(s)
    });

    i.insert("north", |g, u, _| g.dir_func(u, MapDir::North));
    i.insert("south", |g, u, _| g.dir_func(u, MapDir::South));
    i.insert("east", |g, u, _| g.dir_func(u, MapDir::East));
    i.insert("west", |g, u, _| g.dir_func(u, MapDir::West));

    i.insert("ouch", |g, u, _| {
        const PRICK: usize = 5;
        g.players.entry(u).or_default().hurt(PRICK);

        Some(format!("{}", Red("that hurt a surprising amount".into())))
    });

    i.insert("inventory", |g, u, _a| g.list_inventory(u));

    i.insert("", |_, _, _| Some("".to_owned()));

    i.insert("none", |_, _, _| Some(random_insult()));

    i.insert("quit", |_, _, _| return None)
}
