use crate::fight::FightMessage;
use crate::game::Game;
use crate::text::message::{Broadcast, FightAudience};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

type Listener = FightAudience;
type Content = FightMessage;

pub trait MessageHandler {
    fn start(self, caster: Arc<Mutex<Game>>) -> JoinHandle<()>;
}

pub struct MessageReceiver(pub Receiver<(Listener, Content)>);

impl MessageHandler for MessageReceiver {
    fn start(self, caster: Arc<Mutex<Game>>) -> JoinHandle<()> {
        spawn(move || {
            let caster = caster.clone();
            for (aud, msg) in self.0 {
                caster.lock().unwrap().send(&aud, &msg);
            }
        })
    }
}

pub enum DiscreteMessage {
    KillPlayer(u128),
}

pub struct GameActor(pub Receiver<DiscreteMessage>);

impl MessageHandler for GameActor {
    fn start(self, caster: Arc<Mutex<Game>>) -> JoinHandle<()> {
        spawn(move || {
            let caster = caster.clone();
            for dm in self.0 {
                if let DiscreteMessage::KillPlayer(p) = dm {
                    caster.lock().unwrap().kill_player(p);
                }
            }
        })
    }
}
