use crate::map::Locate;
use crate::player::{Player as BarePlayer, Uuid};
use crate::text::Color::{Red, Yellow};

use std::ops::DerefMut;

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use crate::fight::Starter::{Aggressor, Defender};
use crate::item::Describe;

use crate::text::message::{Audience, MessageFormat, Msg};
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
    fn end(&mut self);
}

pub struct BasicFight {
    player_a: Player,
    player_b: Player,
    delay: Duration,
    ended: Arc<Mutex<bool>>,
    sender: Sender<(Audience<u128, Vec<u128>>, Msg<String, String>)>,
}

pub struct FightInfo {
    pub player_a: Player,
    pub player_b: Player,
    pub delay: Duration,
    pub sender: Sender<(Audience<u128, Vec<u128>>, Msg<String, String>)>,
}

impl BasicFight {
    pub fn new(info: FightInfo) -> Arc<Mutex<Self>> {
        let FightInfo {
            player_a,
            player_b,
            delay,
            sender,
        } = info;

        arc_mutex!(Self {
            player_a,
            player_b,
            delay,
            sender,
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

        let mut cl = self.clone();

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
                let FightStatus { ended } = cl.status();
                println!("status: {}", ended);
                if ended {
                    break;
                }

                if a_loc != b_loc {
                    cl.end();
                    break;
                }

                a_loc = pa.lock().unwrap().loc();
                if a_loc != b_loc {
                    cl.end();
                    break;
                }

                fight_logic(
                    &mut cl,
                    &fight_sender,
                    aid,
                    bid,
                    &aname,
                    &bname,
                    pa.clone(),
                    Aggressor,
                )?;

                let FightStatus { ended } = cl.status();
                println!("status: {}", ended);
                if ended {
                    break;
                }

                b_loc = pb.lock().unwrap().loc();
                if a_loc != b_loc {
                    cl.end();
                    break;
                }

                fight_logic(
                    &mut cl,
                    &fight_sender,
                    bid,
                    aid,
                    &bname,
                    &aname,
                    pb.clone(),
                    Defender,
                )?;

                if a_loc != b_loc {
                    cl.end();
                    break;
                }
                thread::sleep(delay);
            }
            cl.end();
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

pub struct FightMessage {}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Starter {
    Aggressor,
    Defender,
}

fn fight_logic(
    cl: &mut Arc<Mutex<BasicFight>>,
    fight_sender: &Sender<(Audience<u128, Vec<u128>>, Msg<String, String>)>,
    aid: u128,
    bid: u128,
    a_name: &str,
    b_name: &str,
    player: Player,
    starter: Starter,
) -> Result<(), String> {
    let mut player = player.lock().unwrap();
    if !player.is_connected() {
        cl.end();
    }

    let ((a_before, a_after), (b_before, b_after)) = match starter {
        Aggressor => (("\n\n", ""), ("\n\n", "")),
        Defender => (("\n", "\n\n > "), ("\n", "\n\n > ")),
    };

    player.hurt(5);
    fight_sender
        .send((
            Audience(aid, vec![bid]),
            Msg {
                s: format!("{}", Yellow(format!("you hit {}", b_name)))
                    .custom_padded(a_before, a_after),
                o: Some(
                    format!("{}", Red(format!("{} hits you", a_name)))
                        .custom_padded(b_before, b_after),
                ),
            },
        ))
        .map_err(|_| format!("player {} write error", aid))?;

    if player.hp() <= 0 {
        cl.end();
    }

    Ok(())
}
