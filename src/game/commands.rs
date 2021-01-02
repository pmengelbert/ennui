use super::item::Direction;
// use super::item::TransferResult::*;
use super::*;
use crate::game::util::random_insult;
// use crate::item::error::Error::*;
use crate::error::EnnuiError::*;
use crate::error::{CmdErr, EnnuiError};
use crate::map::door::{DoorState, Lock, ObstacleState};
use crate::text::message::{Audience, Msg};

pub fn fill_interpreter(i: &mut Interpreter) {
    i.insert("look", |g, u, args| {
        let msg: Cow<'static, str> = match args.len() {
            0 => g.describe_room(u)?.into(),
            1 => {
                let loc = g.loc_of(u)?;
                if let Some(item) = g.describe_item(u, args[0]) {
                    item.into()
                } else if let Some(person) = g.describe_player(loc, u, args[0]) {
                    person.into()
                } else {
                    format!("i don't see {} here...", article(args[0])).into()
                }
            }
            _ => "what you're saying is not clear from context".into(),
        };

        message(u, msg)
    });

    i.insert("take", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let aud = Audience(u, g.rooms.player_ids(loc).except(u));

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
                    Err(err) => match err {
                        Simple(CmdErr::TooHeavy) => {
                            format!("you can't pick up {}. It's too heavy", article(handle))
                        }
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

                match room.get_item_mut(container) {
                    Some(c) => {
                        if let Item::Container(cont) = c {
                            use std::result::Result::*;
                            match cont.get_item(object) {
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

        message(
            aud,
            Msg {
                s: self_msg,
                o: other_msg,
            },
        )
    });

    i.insert("wear", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let aud = Audience(u, g.rooms.player_ids(loc).except(u));

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
                    Err(err) => match err {
                        Simple(CmdErr::NotClothing) => {
                            format!("you can't wear {}!", article(handle))
                        }
                        Simple(CmdErr::ItemNotFound) => {
                            format!("you're not holding {}", article(handle))
                        }
                        Fatal(e) => {
                            return Err(Fatal(format!("[{}]: {}", Red("FATAL".into()), e)));
                        }
                        Msg(m) => m,
                        _ => todo!(),
                    },
                }
            }
            _ => "be more specific. or less specific.".to_owned(),
        };

        message(
            aud,
            Msg {
                s: self_msg,
                o: other_msg,
            },
        )
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

        let others = g.rooms.player_ids(loc).except(u);
        message(
            Audience(u, others),
            Msg {
                s: msg,
                o: other_msg,
            },
        )
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

        let aud = Audience(u, g.rooms.player_ids(loc).except(u));
        let msg = Msg {
            s: msg,
            o: other_msg,
        };

        message(aud, msg)
    });

    i.insert("give", |g, u, a| {
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;

        let mut other_id = vec![];
        let mut other_msg = None;

        let p_msg = if a.len() == 2 {
            let (other, handle) = (a[0], a[1]);

            match g.transfer(u, Some(other), Direction::Give, handle) {
                Ok(h) => {
                    let art = article(&h);

                    other_id.push(g.id_of_in(loc, other)?.uuid());
                    other_msg = Some(format!("{} gives you {}", name, art));

                    format!("you give {} {}", other, art)
                }
                Err(err) => match err {
                    Simple(s) => match s {
                        CmdErr::ItemNotFound => format!("you're not holding {}", article(handle)),
                        CmdErr::TooHeavy => format!(
                            "you fail in your effort. {} is too heavy for them to carry",
                            article(handle)
                        ),
                        CmdErr::PlayerNotFound => {
                            format!("you realize you don't see them here, and you begin to panic")
                        }
                        _ => {
                            return Err(Fatal(format!("GIVE: SHOULD BE UNREACHABLE")));
                        }
                    },
                    EnnuiError::Fatal(e) => return Err(EnnuiError::Fatal(e)),
                    Msg(s) => s,
                    _ => {
                        return Err(Fatal(format!("GIVE: SHOULD BE UNREACHABLE")));
                    }
                },
            }
        } else {
            "E - NUN - CI - ATE".to_owned()
        };

        let aud = Audience(u, other_id);
        message(
            aud,
            Msg {
                s: p_msg,
                o: other_msg,
            },
        )
    });

    i.insert("say", |g, u, a| {
        let msg = a.join(" ");
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let others = g.rooms.player_ids(loc).except(u);

        let aud = Audience(u, others);
        let msg = Msg {
            s: format!("you say '{}'", msg),
            o: Some(format!("{} says '{}'", name, msg)),
        };

        message(aud, msg)
    });

    i.insert("chat", |g, u, a| {
        let statement = a.join(" ");
        let name = g.name_of(u)?.to_owned();
        let aud = Audience(u, g.players.others());

        let msg = Msg {
            s: format!("you chat '{}'", statement),
            o: Some(format!("{} chats '{}'", name, statement)),
        };

        message(aud, msg)
    });

    i.insert("evaluate", |g, u, _| {
        let p = g.players.get(&u)?;

        let mut s = String::new();
        for meter in p.stats() {
            s.push_str(&format!("{:#?}", meter));
        }

        message(u, s)
    });

    i.insert("open", |g, u, a| {
        let loc = g.loc_of(u)?;
        let name = g.name_of(u)?;
        let mut other_msg = None;
        let self_msg = match a.len() {
            0 => format!("ok, what do you want to open?"),
            1 => {
                let rooms = &mut g.rooms;
                let room = rooms.get_mut(&loc)?;

                if room.doors().len() > 1 {
                    format!("which door do you want to open?")
                } else {
                    let door = match room.doors().iter_mut().next() {
                        Some((_, d)) => d,
                        None => return message(u, "there's no door here"),
                    };
                    use std::result::Result::*;
                    match door.unlock(DoorState::Open, std::option::Option::None) {
                        Ok(_) => {
                            println!("doorstate: {}", door.state());
                            other_msg = Some(format!("{} opens a door", name));
                            format!("the door swings open")
                        }
                        Err(err) => match err {
                            DoorState::Locked => format!("that door is locked"),
                            DoorState::Open => format!("it's already open"),
                            DoorState::MagicallySealed => {
                                format!("it's sealed by some unfamiliar magic")
                            }
                            DoorState::PermaLocked => format!("it ain't gonna budge"),
                            _ => format!("wtf"),
                        },
                    }
                }
            }
            2 => todo!(),
            _ => format!("I'm not sure what you're getting at"),
        };

        let aud = Audience(u, g.players_in(loc).except(u));
        let msg = Msg {
            s: self_msg,
            o: other_msg,
        };
        message(aud, msg)
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

                let room = rooms.get_mut(&loc).ok_or(Fatal(format!(
                    "UNABLE TO FIND ROOM {:?} for player {}",
                    loc, u
                )))?; // TODO: fix early exit
                let player = players
                    .get_mut(&u)
                    .ok_or(Fatal(format!("UNABLE TO FIND player {}", u)))?; // TODO: fix early exit

                let num_doors = room.doors().len();
                match num_doors {
                    0 => format!("there's nothing to unlock here"),
                    1 => {
                        match handle.to_lowercase().as_str() {
                            "door" => {
                                let door = match room.doors().iter_mut().next() {
                                    Some((_, d)) => d,
                                    None => return message(u, "there's no door here"),
                                };
                                let mut res = None;

                                for item in player.items_mut().iter_mut() {
                                    if let Item::Key(k) = item {
                                        use std::result::Result::*;
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

        let aud = Audience(u, g.rooms.player_ids(loc).except(u));
        let msg = Msg {
            s: self_msg,
            o: other_msg,
        };

        message(aud, msg)
    });

    i.insert("north", |g, u, _| g.dir_func(u, MapDir::North));
    i.insert("south", |g, u, _| g.dir_func(u, MapDir::South));
    i.insert("east", |g, u, _| g.dir_func(u, MapDir::East));
    i.insert("west", |g, u, _| g.dir_func(u, MapDir::West));

    i.insert("ouch", |g, u, _| {
        const PRICK: usize = 5;
        g.players.entry(u).or_default().hurt(PRICK);

        message(
            u,
            format!("{}", Red("that hurt a surprising amount".into())),
        )
    });

    i.insert("inventory", |g, u, _a| {
        let aud = u;
        let msg = g.list_inventory(u)?;
        message(aud, msg)
    });

    i.insert("", |_, _, _| message(0, ""));

    i.insert("none", |_, u, _| message(u, random_insult()));

    i.insert("quit", |_, _, _| Err(Quit))
}
