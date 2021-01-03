use crate::map::Locate;
use crate::player::{Player as BarePlayer, Uuid};
use crate::text::Color::{Red, Yellow};

use std::io::Write;
use std::ops::DerefMut;

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use crate::item::Describe;
use crate::text::message::{Audience, Msg, MessageFormat};
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
        let cl = self.clone();
        spawn(move || {
            for handle in receiver {
                match handle.join() {
                    Ok(_r) => println!("fight concluded"),
                    Err(e) => {
                        cl.clone().end();
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
            let player_a = pa.clone();
            let player_b = pb.clone();
            loop {
                let a_loc = {
                    let mut a = player_a.lock().unwrap();
                    if !a.is_connected() {
                        cl.end();
                    }

                    let FightStatus { ended } = cl.status();
                    {
                        println!("status: {}", ended);
                        if ended {
                            break;
                        }
                    }

                    a.hurt(5);
                    fight_sender
                        .send((
                            Audience(aid, vec![bid]),
                            Msg {
                                s: format!("{}", Yellow(format!("you hit {}", bname))).custom_padded("\n\n", ""),
                                o: Some(format!("{}", Red(format!("{} hits you", aname))).custom_padded("\n\n", "")),
                            },
                        ))
                        .map_err(|_| String::from("player a write error"))?;
                    if a.hp() <= 0 {
                        cl.end();
                        break;
                    }
                    a.loc()
                };

                let b_loc = {
                    let mut b = player_b.lock().unwrap();
                    if !b.is_connected() {
                        cl.end();
                    }
                    let FightStatus { ended } = cl.status();
                    {
                        println!("status: {}", ended);
                        if ended {
                            break;
                        }
                    }
                    b.hurt(5);
                    fight_sender
                        .send((
                            Audience(bid, vec![aid]),
                            Msg {
                                s: format!("{}", Yellow(format!("you hit {}", aname))).custom_padded("\n", "\n\n > "),
                                o: Some(format!("{}", Red(format!("{} hits you", bname))).custom_padded("\n", "\n\n > ")),
                            },
                        ))
                        .map_err(|_| String::from("player b write error"))?;
                    if b.hp() <= 0 {
                        cl.end();
                        break;
                    }
                    b.loc()
                };
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
