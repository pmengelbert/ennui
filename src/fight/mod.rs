use crate::map::Locate;
use crate::player::{Player as BarePlayer, Uuid};

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
use crate::text::Color::{Red, Yellow};
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
    fn sender(&self) -> Sender<(FightAudience, FightMessage)>;
    fn send_message(&mut self, m: FightMod) -> Result<(), EnnuiError>;
    fn end(&mut self);
}

#[derive(Copy, Clone)]
pub enum FightMod {
    Leave(u128),
}

pub struct BasicFight {
    aggressor: Player,
    defender: Player,
    delay: Duration,
    ended: Arc<Mutex<bool>>,
    audience: PlayerIdList,
    sender: Sender<(FightAudience, FightMessage)>,
    receiver: Option<Receiver<FightMod>>,
}

pub struct FightInfo {
    pub aggressor: Player,
    pub defender: Player,
    pub delay: Duration,
    pub audience: PlayerIdList,
    pub sender: Sender<(FightAudience, FightMessage)>,
    pub receiver: Receiver<FightMod>,
}

impl BasicFight {
    pub fn new(info: FightInfo) -> Arc<Mutex<Self>> {
        let FightInfo {
            aggressor,
            defender,
            delay,
            sender,
            audience,
            receiver,
        } = info;

        arc_mutex!(Self {
            aggressor,
            defender,
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
        begin_fight(self.clone())
    }

    fn status(&self) -> FightStatus {
        let ended = *self.lock().unwrap().ended.lock().unwrap();
        FightStatus { ended }
    }

    fn sender(&self) -> Sender<(FightAudience, FightMessage)> {
        self.lock().unwrap().sender.clone()
    }

    fn send_message(&mut self, m: FightMod) -> Result<(), EnnuiError> {
        use FightMod::*;
        let cl = self.clone();
        let mut cl = cl.lock().unwrap();

        match m {
            Leave(u) => match cl.audience.remove(&u) {
                true => Ok(()),
                false => Err(Simple(PlayerNotFound)),
            },
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

fn begin_fight(basic_fight: Arc<Mutex<BasicFight>>) -> Result<FightStatus, Error> {
    let (sender, receiver) = channel::<JoinHandle<Result<(), String>>>();
    let fight = basic_fight.clone();
    spawn(move || handle_receiver(receiver, fight));

    let mod_receiver = basic_fight
        .lock()
        .unwrap()
        .receiver
        .take()
        .ok_or("Fight: Unable to TAKE receiver".to_owned())?;

    let fight = basic_fight.clone();
    spawn(move || handle_fight_messages(mod_receiver, fight));

    let mut fight = basic_fight.clone();
    let (delay, fight_sender) = {
        let fight = fight.lock().unwrap();
        (fight.delay.clone(), fight.sender.clone())
    };

    let (player_a, a_id, a_name) = extract(&fight, Aggressor);
    let (player_b, b_id, b_name) = extract(&fight, Defender);

    sender.send(spawn(move || {
        handle_fight(
            &mut fight,
            delay,
            &fight_sender,
            player_a,
            a_id,
            a_name,
            player_b,
            b_id,
            b_name,
        )
    }))?;

    Ok(FightStatus { ended: true })
}

fn handle_receiver(
    receiver: Receiver<JoinHandle<Result<(), String>>>,
    fight: Arc<Mutex<BasicFight>>,
) {
    for handle in receiver {
        match handle.join() {
            Ok(_r) => println!("fight concluded"),
            Err(e) => {
                fight.clone().end();
                println!("[{}]: {:?}", "ERROR".color(Red), e)
            }
        }
    }
}

fn handle_fight_messages(mod_receiver: Receiver<FightMod>, mut fight: Arc<Mutex<BasicFight>>) {
    for modification in mod_receiver {
        let res = fight.send_message(modification);

        if let Err(e) = res {
            println!("{:?}", e);
        }
    }
}

fn extract(fight: &Arc<Mutex<BasicFight>>, player: Starter) -> (Player, u128, String) {
    use Starter::*;
    let (pa, aid, aname) = {
        let fight = fight.lock().unwrap();
        let player_info = match player {
            Aggressor => fight.aggressor.clone(),
            Defender => fight.defender.clone(),
        };
        let (id, name) = {
            let unlocked = player_info.lock().unwrap();
            (unlocked.uuid(), unlocked.name())
        };
        (player_info, id, name)
    };
    (pa, aid, aname)
}

fn handle_fight(
    mut fight: &mut Arc<Mutex<BasicFight>>,
    delay: Duration,
    fight_sender: &Sender<(FightAudience, FightMessage)>,
    pa: Player,
    aid: u128,
    aname: String,
    pb: Player,
    bid: u128,
    bname: String,
) -> Result<(), String> {
    let mut a_loc = pa.loc();
    let mut b_loc = pb.loc();
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

        a_loc = pa.loc();
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

        b_loc = pb.loc();
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
                s: format!("you hit {}", b_name)
                    .color(Yellow)
                    .custom_padded(before, after)
                    .into(),
                obj: Some(
                    format!("{} hits you", a_name)
                        .color(Red)
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
