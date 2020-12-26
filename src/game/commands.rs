use super::item::Direction;

use super::*;
use crate::item::error::Error::*;
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
                match g.transfer(u, None, Direction::Take, handle) {
                    Ok(handle) => {
                        other_msg =
                            Some(format!("{} picks up a {}", name, article(&handle.clone())));
                        format!("you take the {}", handle)
                    }
                    Err(err) => match &*err {
                        TooHeavy(s) => format!("you can't pick up {}. It's too heavy", article(&s)),
                        _ => format!("you don't see {} here", article(&handle)),
                    },
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

        use crate::item::error::Error::*;

        let mut other_msg = None;
        let self_msg = match a.len() {
            0 => "there seems to be an error".to_owned(),
            1 => {
                let handle = a[0];
                match g.transfer(u, None, Direction::Wear, handle) {
                    Ok(item_name) => {
                        other_msg = Some(format!("{} puts on {}", name, article(&item_name)));
                        format!("you wear the {}", handle)
                    }
                    Err(err) => match &*err {
                        Clothing(s) => format!("you can't wear {}!", article(s)),
                        s => format!("you're not holding {}", article(s.safe_unwrap())),
                    },
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

            match g.transfer(u, None, Direction::Remove, handle) {
                Ok(handle) => {
                    other_msg = Some(format!("{} takes off {}", name, article(&handle)));
                    format!("you take off the {}", handle)
                }
                Err(err) => match err {
                    _ => format!("you're not wearing {}", article(&handle)),
                },
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

    i.insert("drop", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let mut other_msg = None;

        let msg = if a.len() == 1 {
            let handle = a[0];
            match g.transfer(u, None, Direction::Drop, handle) {
                Ok(handle) => {
                    other_msg = Some(format!("{} drops {}", name, article(&handle)));
                    format!("you drop the {}", handle)
                }
                Err(_) => format!("you don't see {} here", article(handle)),
            }
        } else {
            "be more specific. or less specific.".to_owned()
        };

        let others = loc.player_ids(&g)?;
        let aud = Audience(u, &others);
        let msg = Msg {
            s: msg,
            o: other_msg,
        };

        g.send(aud, msg);

        Some("".into())
    });

    i.insert("give", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;

        let mut other_id = None;
        let mut other_msg = None;

        let p_msg = if a.len() == 2 {
            let (other, handle) = (a[0], a[1]);

            match g.transfer(u, Some(other), Direction::Give, handle) {
                Ok(h) => {
                    let art = article(&h);

                    other_id = Some(vec![loc.player_by_name(&g, other)?.uuid()]);
                    other_msg = Some(format!("{} gives you {}", name, art));

                    format!("you give {} {}", other, art)
                }
                Err(err) => match &*err {
                    ItemNotFound(s) => format!("you're not holding {}", article(&s)),
                    PlayerNotFound(s) => {
                        format!("now where did {} go? you don't see them here...", s)
                    }
                    Clothing(_) => format!("they must not like the look of it"),
                    TooHeavy(_) => format!("they can't hold that! it's too heavy"),
                },
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

        g.send(aud, msg);
        Some("".into())
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
        g.send(u, &s);

        Some("".into())
    });

    i.insert("north", |g, u, _| g.dir_func(u, MapDir::North));
    i.insert("south", |g, u, _| g.dir_func(u, MapDir::South));
    i.insert("east", |g, u, _| g.dir_func(u, MapDir::East));
    i.insert("west", |g, u, _| g.dir_func(u, MapDir::West));

    i.insert("ouch", |g, u, _| {
        const PRICK: usize = 5;
        g.players.entry(u).or_default().hurt(PRICK);

        g.send(
            u,
            format!("{}", Red("that hurt a surprising amount".into())),
        );
        Some("".into())
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
