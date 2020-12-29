use super::item::Direction;

use super::*;
use crate::item::error::Error::*;
use crate::map::door::{DoorState, Lock, ObstacleState};
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
            2 => {
                let rooms = &mut g.rooms;
                let players = &mut g.players;
                let (object, container) = (a[0], a[1]);
                // find container
                let room = rooms.get_mut(&loc)?;
                let player = players.get_mut(&u)?;

                match room.get_mut(container) {
                    Some(c) => {
                        if let Item::Container(cont) = c {
                            match cont.get(object) {
                                Some(_) => match cont.transfer(player, object) {
                                    Ok(handle) => {
                                        other_msg = Some(format!(
                                            "{} takes {} from {}",
                                            name,
                                            article(object),
                                            article(container),
                                        ));
                                        format!("you take the {}", handle)
                                    }
                                    Err(_) => {
                                        format!("you somehow failed at the simplest of tasks")
                                    }
                                },
                                None => format!(
                                    "you don't see {} in the {}",
                                    article(object),
                                    container
                                ),
                            }
                        } else {
                            format!("{} is not a container!", article(container))
                        }
                    }
                    None => format!("you don't see {} here", article(container)),
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

    i.insert("open", |g, u, a| {
        use crate::map::door::DoorState::*;
        let loc = g.loc_of(u)?;
        let name = g.name_of(u)?;
        let mut other_msg = std::option::Option::None;
        let self_msg = match a.len() {
            0 => format!("ok, what do you want to open?"),
            1 => {
                let rooms = &mut g.rooms;
                let room = rooms.get_mut(&loc)?;

                if room.doors().len() > 1 {
                    format!("which door do you want to open?")
                } else {
                    let (_, door) = room.doors().iter_mut().next()?;
                    match door.unlock(Open, std::option::Option::None) {
                        Ok(_) => {
                            println!("doorstate: {}", door.state());
                            other_msg = Some(format!("{} opens a door", name));
                            format!("the door swings open")
                        }
                        Err(err) => match err {
                            Locked => format!("that door is locked"),
                            Open => format!("it's already open"),
                            MagicallySealed => format!("it's sealed by some unfamiliar magic"),
                            PermaLocked => format!("it ain't gonna budge"),
                            _ => format!("wtf"),
                        },
                    }
                }
            }
            2 => todo!(),
            _ => format!("I'm not sure what you're getting at"),
        };

        let aud = Audience(u, loc.player_ids(&g.rooms)?);
        let msg = Msg {
            s: self_msg,
            o: other_msg,
        };
        g.send(aud, msg);

        Some("".into())
    });

    i.insert("unlock", |g, u, a| {
        let loc = g.loc_of(u)?;
        let name = g.name_of(u)?;

        let mut other_msg = None;
        let self_msg = match a.len() {
            0 => format!("ok, what do you want to unlock?"),
            1 => {
                let handle = a[0];
                let rooms = &mut g.rooms;
                let players = &mut g.players;

                let room = rooms.get_mut(&loc)?; // TODO: fix early exit
                let player = players.get_mut(&u)?;

                let num_doors = room.doors().len();
                match num_doors {
                    0 => format!("there's nothing to unlock here"),
                    1 => {
                        match handle.to_lowercase().as_str() {
                            "door" => {
                                let (_, door) = room.doors().iter_mut().next()?;
                                let mut res = None;

                                for item in player.items_mut().iter_mut() {
                                    if let Item::Key(k) = item {
                                        match door.unlock(DoorState::Closed, Some(&**k)) {
                                            Ok(()) => {
                                                println!("door state: {}", door.state());
                                                other_msg =
                                                    Some(format!("{} unlocks a door", name));
                                                res = Some(());
                                                break;
                                            }
                                            Err(_) => continue,
                                        }
                                    }
                                }
                                match res {
                                    Some(()) => format!("*click*"),
                                    None => match door.state() {
                                        DoorState::Locked => {
                                            format!("you don't have the proper key")
                                        }
                                        DoorState::Closed => format!("you've already unlocked it"),
                                        DoorState::Open => format!("it's already open"),
                                        DoorState::MagicallySealed => {
                                            format!("it's sealed by some unfamiliar magic")
                                        }
                                        DoorState::PermaLocked => format!("it ain't gonna budge"),
                                        _ => format!("wtf"),
                                    },
                                }
                            }
                            // TODO: handle other unlockable items (such as chests) here
                            _ => format!("I'm not sure that you can even unlock that"),
                        }
                    }
                    _ => format!("that's all greek to me"),
                }
            }
            _ => format!("that's pretty much gobbledygook to me"),
        };

        let aud = Audience(u, loc.player_ids(&g.rooms)?);
        let msg = Msg {
            s: self_msg,
            o: other_msg,
        };

        g.send(aud, msg);

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
