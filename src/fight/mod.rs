use crate::map::Locate;
use crate::player::{Player as BarePlayer, Uuid};
use crate::text::Color::{Red, Yellow};

use std::ops::DerefMut;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::fight::Starter::{Aggressor, Defender};
use crate::item::{Attribute, Describe};

use crate::error::CmdErr::PlayerNotFound;
use crate::error::EnnuiError;
use crate::error::EnnuiError::Simple;
use crate::player::list::PlayerIdList;
use crate::player::PlayerStatus::Dead;
use crate::text::message::{FightAudience, Message, MessageFormat};
use std::borrow::{Borrow, Cow};
use std::error::Error as StdError;
use std::thread;
use std::thread::{spawn, JoinHandle};
use std::time::Duration;

type Player = Arc<Mutex<BarePlayer>>;
type Error = Box<dyn StdError>;

pub trait Fight {
    fn begin(&mut self) -> Result<FightStatus, Error>;
    fn status(&self) -> FightStatus;
    fn sender(&self) -> Sender<FightMessage>;
    fn send_message(&mut self, m: FightMod) -> Result<(), EnnuiError>;
    fn end(&mut self);
}

#[derive(Copy, Clone)]
pub enum FightMod {
    Leave(u128),
}

pub struct BasicFight {
    player_a: Player,
    player_b: Player,
    delay: Duration,
    ended: Arc<Mutex<bool>>,
    audience: PlayerIdList,
    sender: Sender<(FightAudience, FightMessage)>,
    receiver: Option<Receiver<FightMod>>,
}

pub struct FightInfo {
    pub player_a: Player,
    pub player_b: Player,
    pub delay: Duration,
    pub audience: PlayerIdList,
    pub sender: Sender<(FightAudience, FightMessage)>,
    pub receiver: Receiver<FightMod>,
}

impl BasicFight {
    pub fn new(info: FightInfo) -> Arc<Mutex<Self>> {
        let FightInfo {
            player_a,
            player_b,
            delay,
            sender,
            audience,
            receiver,
        } = info;

        arc_mutex!(Self {
            player_a,
            player_b,
            delay,
            sender,
            audience,
            receiver: Some(receiver),
            ended: arc_mutex!(false),
        })
    }
}

impl Fight for Arc<Mutex<BasicFight>> {
    fn begin(&mut self) -> Result<FightStatus, Error> {
        let (sender, receiver) = channel::<JoinHandle<Result<(), String>>>();
        let fight = self.clone();
        spawn(move || {
            for handle in receiver {
                match handle.join() {
                    Ok(_r) => println!("fight concluded"),
                    Err(e) => {
                        fight.clone().end();
                        println!("[{}]: {:?}", Red("ERROR".to_owned()), e)
                    }
                }
            }
        });

        let mod_receiver = self
            .lock()
            .unwrap()
            .receiver
            .take()
            .ok_or("OH NO".to_owned())?;
        let mut fight = self.clone();
        spawn(move || {
            for modification in mod_receiver {
                let res = fight.send_message(modification);

                if let Err(e) = res {
                    println!("{:?}", e);
                }
            }
        });

        let mut fight = self.clone();

        let pa = self.lock().unwrap().player_a.clone();
        let pb = self.lock().unwrap().player_b.clone();
        let delay = self.lock().unwrap().delay.clone();
        let fight_sender = self.lock().unwrap().sender.clone();
        let aid = pa.lock().unwrap().uuid();
        let bid = pb.lock().unwrap().uuid();

        let aname = pa.lock().unwrap().name().to_owned();
        let bname = pb.lock().unwrap().name().to_owned();

        sender.send(spawn(move || {
            let mut a_loc = pa.lock().unwrap().loc();
            let mut b_loc = pb.lock().unwrap().loc();
            loop {
                let audience: Vec<u128> = fight.lock().unwrap().audience.iter().cloned().collect();

                let FightStatus { ended } = fight.status();
                println!("status: {}", ended);
                if ended {
                    break;
                }

                if a_loc != b_loc {
                    fight.end();
                    break;
                }

                a_loc = pa.lock().unwrap().loc();
                if a_loc != b_loc {
                    fight.end();
                    break;
                }

                fight_logic(
                    &mut fight,
                    &fight_sender,
                    aid,
                    bid,
                    &aname,
                    &bname,
                    pa.clone(),
                    Aggressor,
                    &audience,
                )?;

                let FightStatus { ended } = fight.status();
                println!("status: {}", ended);
                if ended {
                    break;
                }

                b_loc = pb.lock().unwrap().loc();
                if a_loc != b_loc {
                    fight.end();
                    break;
                }

                fight_logic(
                    &mut fight,
                    &fight_sender,
                    bid,
                    aid,
                    &bname,
                    &aname,
                    pb.clone(),
                    Defender,
                    &audience,
                )?;

                if a_loc != b_loc {
                    fight.end();
                    break;
                }
                thread::sleep(delay);
            }
            fight.end();
            Ok(())
        }))?;
        let ended = *self.lock().unwrap().ended.lock().unwrap();
        Ok(FightStatus { ended })
    }

    fn status(&self) -> FightStatus {
        let ended = *self.lock().unwrap().ended.lock().unwrap();
        FightStatus { ended }
    }

    fn sender(&self) -> Sender<FightMessage> {
        unimplemented!()
    }

    fn send_message(&mut self, m: FightMod) -> Result<(), EnnuiError> {
        use FightMod::*;
        let cl = self.clone();
        let mut cl = cl.lock().unwrap();

        match m {
            Leave(u) => {
                match cl.audience.remove(&u) {
                    true => Ok(()),
                    false => Err(Simple(PlayerNotFound)),
                }
            }
        }
    }

    fn end(&mut self) {
        let cl = self.clone();
        let cl = cl.lock().unwrap();
        let mut x = cl.ended.lock().unwrap();
        let x = x.deref_mut();
        *x = true;
    }
}

pub struct FightStatus {
    pub ended: bool,
}

pub struct FightMessage {
    pub s: Cow<'static, str>,
    pub obj: Option<Cow<'static, str>>,
    pub oth: Option<Cow<'static, str>>,
}

impl Message for FightMessage {
    fn to_self(&self) -> String {
        let mut s = String::new();
        s.push_str(self.s.borrow());
        s
    }

    fn to_object(&self) -> Option<String> {
        let mut s = String::new();
        s.push_str(self.obj.as_ref()?.borrow());
        Some(s)
    }

    fn to_others(&self) -> Option<String> {
        let mut s = String::new();
        s.push_str(self.oth.as_ref()?.borrow());
        Some(s)
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Starter {
    Aggressor,
    Defender,
}

fn fight_logic(
    cl: &mut Arc<Mutex<BasicFight>>,
    fight_sender: &Sender<(FightAudience, FightMessage)>,
    aid: u128,
    bid: u128,
    a_name: &str,
    b_name: &str,
    player: Player,
    starter: Starter,
    audience: &[u128],
) -> Result<(), String> {
    let mut player = player.lock().unwrap();
    if !player.is_connected() {
        cl.end();
    }

    let (before, after) = match starter {
        Aggressor => ("\n\n", ""),
        Defender => ("\n", "\n\n > "),
    };

    player.hurt(25);
    fight_sender
        .send((
            FightAudience(aid, bid, audience.to_vec()),
            FightMessage {
                s: format!("{}", Yellow(format!("you hit {}", b_name)))
                    .custom_padded(before, after)
                    .into(),
                obj: Some(
                    format!("{}", Red(format!("{} hits you", a_name)))
                        .custom_padded(before, after)
                        .into(),
                ),
                oth: Some(
                    format!("{} hits {}", a_name, b_name)
                        .custom_padded(before, after)
                        .into(),
                ),
            },
        ))
        .map_err(|_| format!("player {} write error", aid))?;

    if player.hp() <= 0 {
        player.set_attr(Dead);
        cl.end();
    }

    Ok(())
}
