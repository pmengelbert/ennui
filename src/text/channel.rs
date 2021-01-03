use std::sync::mpsc::Receiver;
use crate::text::message::{Messenger, Message, Broadcast, Audience, Msg};
use std::thread::{JoinHandle, spawn};
use std::sync::{Mutex, Arc};

type Listener = Audience<u128, Vec<u128>>;
type Content = Msg<String, String>;

pub trait MessageHandler {
    fn start<T: Broadcast + Send + 'static>(self, caster: Arc<Mutex<T>>) -> JoinHandle<()>;
}

pub struct MessageReceiver(pub Receiver<(Listener, Content)>);

impl MessageHandler for MessageReceiver {
    fn start<T: Broadcast + Send + 'static>(mut self, caster: Arc<Mutex<T>>) -> JoinHandle<()> {
        spawn(move || {
            let caster = caster.clone();
            for (aud, msg) in self.0 {
                caster.lock().unwrap().send(&aud, &msg);
            }
        })
    }
}

