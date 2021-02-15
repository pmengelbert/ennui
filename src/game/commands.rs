use super::item::Direction;
use super::*;
use crate::db::recipe_to_item;
use crate::error::EnnuiError::*;
use crate::error::{CmdErr, EnnuiError};
use crate::game::util::random_insult;
use crate::obstacle::door::{Door, DoorState, Lock, ObstacleState};
use crate::text::message::{Audience, Msg};

use crate::fight::{BasicFight, Fight, FightInfo, FightMod};
use crate::soul::recipe::Recipe;
use crate::text::Color::{Green, Red, Yellow};
use std::convert::TryInto;
use std::ops::DerefMut;
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn fill_interpreter(i: &mut Interpreter) {
    i.insert("look", |g, u, args| {
        eprintln!("[{}]: made it to handler", "SUCCESS".color(Green));
        eprintln!("in file {} on line number {}", file!(), line!());

        let args: Vec<_> = args.iter().filter(|&&a| a != "at").collect();
        let msg: Cow<'static, str> = match args.len() {
            0 => {
                eprintln!("[{}]: made it to case", "SUCCESS".color(Green));
                eprintln!("in file {} on line number {}", file!(), line!());

                g.describe_room(u)?.into()
            }
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
        let a: Vec<_> = a.iter().filter(|&&a| a != "from").collect();
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
                        other_msg = Some(format!("{} picks up a {}", name, article(&handle)));
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
                let player = g.get_player(u)?;
                let room = g.get_room_mut(loc)?;
                let (object, container) = (*a[0], *a[1]);

                match room.get_item_mut(container.into()) {
                    Some(c) => {
                        if let Item::Container(cont) = c {
                            use std::result::Result::*;
                            match cont.get_item_mut(object.into()) {
                                Some(_) => match cont
                                    .transfer(player.lock().unwrap().deref_mut(), object)
                                {
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
                                        "you somehow failed at the simplest of tasks".to_owned()
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
                            return Err(Fatal(format!("[{}]: {}", "FATAL".color(Red), e)));
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
                Err(_) => format!("you're not wearing {}", article(&handle)),
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
            let (handle, other) = (a[0], a[1]);

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
                        CmdErr::PlayerNotFound => "there's no-one by that name here".to_owned(),
                        _ => {
                            return Err(fatal("GIVE: SHOULD BE UNREACHABLE"));
                        }
                    },
                    EnnuiError::Fatal(e) => return Err(EnnuiError::Fatal(e)),
                    Msg(s) => s,
                    _ => {
                        return Err(fatal("GIVE: SHOULD BE UNREACHABLE"));
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
        eprintln!("MADE IT");
        eprintln!("in file {} on line number {}", file!(), line!());

        let msg = a.join(" ");
        let name = g.name_of(u)?;
        let loc = g.loc_of(u)?;
        let others = g.rooms.player_ids(loc).except(u);
        eprintln!("others: {:?}", others);
        eprintln!("in file {} on line number {}", file!(), line!());

        let aud = Audience(u, others);
        let msg = Msg {
            s: format!("you say '{}'", msg),
            o: Some(format!("{} says '{}'", name, msg)),
        };

        message(aud, msg)
    });

    i.insert("chat", |g, u, a| {
        let statement = a.join(" ");
        let name = g.name_of(u)?;
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
        for meter in p.lock().unwrap().stats() {
            s.push_str(&format!("{:#?}", meter));
        }

        message(u, s)
    });

    i.insert("open", |g, u, a| {
        let loc = g.loc_of(u)?;
        let name = g.name_of(u)?;
        let mut other_msg = None;

        let rooms = &mut g.rooms;
        let room = rooms.get_mut(&loc)?;

        let self_msg = match a.len() {
            0 => "ok, what do you want to open?".to_owned(),
            1 => {
                if room.doors().len() > 1 {
                    "which door do you want to open?".to_owned()
                } else {
                    let door = match room.doors().iter_mut().next() {
                        Some((_, d)) => d,
                        None => return message(u, "there's no door here"),
                    };

                    try_door_open(&name, &mut other_msg, door)
                }
            }
            2 => {
                let dir: MapDir = a[1].into();
                let door = match room.doors().get_mut(&dir) {
                    Some(d) => d,
                    None => return message(u, "there's no door in that direction"),
                };

                try_door_open(&name, &mut other_msg, door)
            }
            _ => "I'm not sure what you're getting at".to_owned(),
        };

        let aud = Audience(u, room.players().except(u));
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
            0 => "ok, what do you want to unlock?".to_owned(),
            1 => {
                let handle = a[0];

                let player = g.get_player(u)?;

                let room = g.get_room_mut(loc)?;

                let num_doors = room.doors().len();
                match num_doors {
                    0 => "there's nothing to unlock here".to_owned(),
                    1 => {
                        match handle.to_lowercase().as_str() {
                            "door" => {
                                let door = match room.doors().iter_mut().next() {
                                    Some((_, d)) => d,
                                    None => return message(u, "there's no door here"),
                                };
                                try_door_unlock(name, &mut other_msg, player, door)
                            }
                            // TODO: handle other unlockable items (such as chests) here
                            _ => "I'm not sure that you can even unlock that".to_owned(),
                        }
                    }
                    _ => "that's all greek to me".to_owned(),
                }
            }
            2 => {
                let player = g.get_player(u)?;
                let room = g.get_room_mut(loc)?;

                let dir: MapDir = a[1].into();
                let door = match room.doors().get_mut(&dir) {
                    Some(d) => d,
                    None => return message(u, "there's no door in that direction"),
                };

                try_door_unlock(name, &mut other_msg, player, door)
            }
            _ => "that's pretty much gobbledygook to me".to_owned(),
        };

        let aud = Audience(u, g.rooms.player_ids(loc).except(u));
        let msg = Msg {
            s: self_msg,
            o: other_msg,
        };

        message(aud, msg)
    });

    i.insert("hit", |g, u, a| {
        let loc = g.loc_of(u)?;

        if !a.is_empty() {
            let object = a[0];

            let other_id = {
                match g.id_of_in(loc, object) {
                    Some(p) if p == u => {
                        return message(
                            u,
                            "violence against the self is all too common. i am here to stop you.",
                        );
                    }
                    Some(p) => p,
                    None => return message(u, format!("you don't see {} here", object)),
                }
            };

            let sender = g.clone_fight_sender()?;

            let aggressor = g.get_player(u)?;
            let defender = g.get_player(other_id)?;

            let audience = g.rooms.player_ids(loc).except(u).except(other_id);
            let (mod_sender, receiver) = channel::<FightMod>();

            for p in audience.iter().filter_map(|&id| Some(g.get_player(id)?)) {
                p.lock().unwrap().assign_fight_sender(mod_sender.clone())
            }

            let discrete_sender = g.discrete_sender.as_ref().expect("HANDLE ME").clone();

            let mut fight = BasicFight::new(FightInfo {
                defender,
                aggressor,
                delay: Duration::new(1, 0),
                audience,
                sender,
                discrete_sender,
                receiver,
            });

            match fight.begin() {
                Ok(_) => (),
                Err(e) => return Err(fatal(&format!("problem happened with the fight: {:?}", e))),
            };
        }
        message(u, "oh no")
    });

    i.insert("sleep", |g, u, _| {
        let loc = g.loc_of(u)?;
        let p = g.get_player(u)?;
        let name = g.name_of(u)?;
        p.lock().unwrap().set_attr(Asleep);

        let others = g.rooms.player_ids(loc).except(u);
        let aud = Audience(u, others);
        let msg = Msg {
            s: String::from("you lay down and fall asleep"),
            o: Some(format!("{} goes to sleep", name)),
        };
        message(aud, msg)
    });

    i.insert("wake", |g, u, _| {
        let loc = g.loc_of(u)?;
        let mut p = g.get_player(u)?;
        let name = g.name_of(u)?;
        p.unset_attr(Asleep);
        p.set_attr(Sitting);

        let others = g.rooms.player_ids(loc).except(u);
        let aud = Audience(u, others);
        let msg = Msg {
            s: String::from("you sit up"),
            o: Some(format!("{} sits up", name)),
        };
        message(aud, msg)
    });

    i.insert("stand", |g, u, _| {
        let loc = g.loc_of(u)?;
        let mut p = g.get_player(u)?;
        let name = g.name_of(u)?;
        p.unset_attr(Sitting);

        let others = g.rooms.player_ids(loc).except(u);
        let aud = Audience(u, others);
        let msg = Msg {
            s: String::from("you stand up"),
            o: Some(format!("{} stands up", name)),
        };
        message(aud, msg)
    });

    i.insert("north", |g, u, _| g.dir_func(u, MapDir::North));
    i.insert("south", |g, u, _| g.dir_func(u, MapDir::South));
    i.insert("east", |g, u, _| g.dir_func(u, MapDir::East));
    i.insert("west", |g, u, _| g.dir_func(u, MapDir::West));

    i.insert("inventory", |g, u, _a| {
        let aud = u;
        let msg = g.list_inventory(u)?;
        message(aud, msg)
    });

    i.insert("who", |g, u, _| {
        let template = "PLAYERS\n-------";

        let mut msg = String::from(template);
        g.players
            .iter()
            .filter_map(|(_, p)| {
                let conditions = {
                    let q = p.lock().unwrap();
                    q.uuid() != u && matches!(&*q, PlayerType::Human(_))
                };

                if conditions {
                    Some(p.name().color(Yellow))
                } else {
                    None
                }
            })
            .for_each(|s| {
                msg.push('\n');
                msg.push_str(&s);
            });

        message(u, msg)
    });

    i.insert("help", |_, u, a| {
        let mut db = match crate::db::DB::new() {
            Ok(db) => db,
            Err(e) => {
                eprintln!("{}", e);
                print_err(fatal("db problem"));
                return message(u, "There was an error with the database. Please file an issue at github.com/pmengelbert/ennui, or email peter@engelbert.dev");
            }
        };

        let s: String = match a.len() {
            1 => db.helpfile(a[0]).unwrap_or_else(|e| {
                eprintln!("{}", e);
                print_err(fatal("dbproblem"));
                format!("There was an error with the database. Please file an issue at github.com/pmengelbert/ennui, or email peter@engelbert.dev")
            }),
            0 => db.helpfile("commands").unwrap_or_else(|e| {
                eprintln!("{}", e);
                print_err(fatal("dbproblem"));
                format!("There was an error with the database. Please file an issue at github.com/pmengelbert/ennui, or email peter@engelbert.dev")
            }),
            _ => "Usage: help <command>".into(),
        };

        message(u, s)
    });

    i.insert("combine", |g, u, a| {
        let r: Recipe = match a.try_into() {
            Ok(r) => r,
            Err(_) => return message(u, "see 'help recipe' for more information"),
        };

        let i = match recipe_to_item(&r) {
            Ok(i) => i,
            Err(e) => {
                print_err(fatal(&format!("{}", e)));
                return message(u, "that recipe doesn't exist!");
            }
        };

        let p = match g.get_player(u) {
            Ok(p) => p,
            Err(_) => return Err(fatal("player does not exist")),
        };

        let mut p = p.lock().unwrap();

        let name = i.name();
        let souls = p.souls_mut();
        if !souls.process_recipe(&r) {
            return message(u, "you don't have that combination of souls!");
        };

        p.insert_item(i);

        message(u, format!("you have created {}", article(&name)))
    });

    i.insert("souls", |g, u, _| {
        let mut p = match g.get_player(u) {
            Ok(p) => p,
            Err(_) => todo!(),
        };

        let mut ret = String::with_capacity(512);

        let p = p.lock().unwrap();
        let list = p.souls().list();

        match list.split_last() {
            Some((last, list)) => {
                for s in list {
                    ret.push_str(&s.name());
                    ret.push('\n');
                }
                ret.push_str(&last.name());
            }
            None => (),
        };

        message(u, ret)
    });

    i.insert("", |_, _, _| message(0, ""));

    i.insert("none", |_, u, _| message(u, random_insult()));

    i.insert("quit", |_, _, _| Err(Quit))
}

fn try_door_unlock(
    name: String,
    other_msg: &mut Option<String>,
    player: Arc<Mutex<PlayerType>>,
    door: &mut Door,
) -> String {
    let mut res = None;

    for item in player.lock().unwrap().list().iter() {
        if let Item::Key(k) = item {
            use std::result::Result::*;
            match door.unlock(DoorState::Closed, Some(k.as_ref())) {
                Ok(()) => {
                    *other_msg = Some(format!("{} unlocks a door", name));
                    res = Some(());
                    break;
                }
                Err(_) => continue,
            }
        }
    }
    match res {
        Some(()) => "*click*",
        None => match door.state() {
            DoorState::Locked => "you don't have the proper key",
            DoorState::Closed => "you've already unlocked it",
            DoorState::Open => "it's already open",
            DoorState::MagicallySealed => "it's sealed by some unfamiliar magic",
            DoorState::PermaLocked => "it ain't gonna budge",
            _ => "wtf",
        },
    }
    .to_owned()
}

fn try_door_open(name: &str, other_msg: &mut Option<String>, door: &mut Door) -> String {
    match door.unlock(DoorState::Open, std::option::Option::None) {
        Ok(_) => {
            *other_msg = Some(format!("{} opens a door", name));
            "the door swings open".to_owned()
        }
        Err(err) => match err {
            DoorState::Locked => "that door is locked".to_owned(),
            DoorState::Open => "it's already open".to_owned(),
            DoorState::MagicallySealed => "it's sealed by some unfamiliar magic".to_owned(),
            DoorState::PermaLocked => "it ain't gonna budge".to_owned(),
            _ => "wtf".to_owned(),
        },
    }
}
