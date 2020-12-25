use super::*;
use crate::text::message::{Audience, Msg};

pub fn fill_interpreter(i: &mut Interpreter) {
    i.insert("look", |g, u, args| {
        let msg = match args.len() {
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
            _ => "what you're saying is not clear from context".into(),
        };

        g.send(u, msg);

        Some("".into())
    });

    i.insert("take", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let aud = Audience(u, loc.player_ids(&g.rooms)?);

        let mut other_msg = None;
        let self_msg = match a.len() {
            0 => "there seems to be an error".to_owned(),
            1 => {
                let handle = a[0];
                if g.transfer(u, None, Direction::Take, handle).is_ok() {
                    other_msg = Some(format!("{} picks up a {}", name, article(handle)));
                    format!("you take the {}", handle)
                } else {
                    format!("you don't see {} here", article(handle))
                }
            }
            _ => "be more specific. or less specific.".to_owned(),
        };

        g.send(
            aud,
            Msg {
                s: self_msg,
                o: other_msg,
            },
        );

        Some("".into())
    });

    i.insert("wear", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let aud = Audience(u, loc.player_ids(&g.rooms)?);

        let mut other_msg = None;
        let self_msg = match a.len() {
            0 => "there seems to be an error".to_owned(),
            1 => {
                let handle = a[0];
                if let Ok(_) = g.transfer(u, None, Direction::Wear, handle) {
                    other_msg = Some(format!("{} puts on {}", name, article(handle)));
                    format!("you wear the {}", handle)
                } else {
                    format!("you're not holding {}", article(handle))
                }
            }
            _ => "be more specific. or less specific.".to_owned(),
        };

        g.send(
            aud,
            Msg {
                s: self_msg,
                o: other_msg,
            },
        );

        Some("".into())
    });

    i.insert("remove", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let mut other_msg = None;

        let msg = if a.len() == 1 {
            let handle = a[0];
            let art = article(handle);
            if let Ok(_) = g.transfer(u, None, Direction::Remove, handle) {
                other_msg = Some(format!("{} takes off {}", name, art));
                format!("you take off the {}", handle)
            } else {
                format!("you're not wearing {}", art)
            }
        } else {
            "be more specific. or less specific.".to_owned()
        };

        let others = loc.player_ids(&g)?;
        g.send(
            Audience(u, &others),
            Msg {
                s: msg,
                o: other_msg,
            },
        );

        Some("".into())
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

    i.insert("give", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;

        let mut other_id = None;
        let mut other_msg = None;

        let p_msg = if a.len() == 2 {
            let (other, handle) = (a[0], a[1]);
            let art = article(handle);

            other_id = Some(vec![loc.player_by_name(&g, other)?.uuid()]);

            if g.transfer(u, Some(other), Direction::Give, handle).is_ok() {
                other_msg = Some(format!("{} gives you {}", name, art));
                format!("you give {} {}", other, art)
            } else {
                "that person or thing isn't here".to_owned()
            }
        } else {
            "E - NUN - CI - ATE".to_owned()
        };

        let aud = Audience(u, other_id);
        g.send(
            aud,
            Msg {
                s: p_msg,
                o: other_msg,
            },
        );

        Some("".into())
    });

    i.insert("say", |g, u, a| {
        let message = a.join(" ");
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let others = loc.player_ids(&g.rooms)?;

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
        let aud = Audience(u, g.players.others());

        let msg = Msg {
            s: format!("you chat '{}'", statement),
            o: Some(format!("{} chats '{}'", name, statement)),
        };

        g.send(aud, msg);

        Some("".into())
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

    i.insert("inventory", |g, u, _a| {
        let aud = u;
        let msg = g.list_inventory(u)?;
        g.send(aud, msg);
        Some("".into())
    });

    i.insert("", |_, _, _| Some("".to_owned()));

    i.insert("none", |_, _, _| Some(random_insult()));

    i.insert("quit", |_, _, _| return None)
}
