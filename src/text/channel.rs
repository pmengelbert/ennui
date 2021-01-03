use crate::text::message::{Audience, Broadcast, Msg};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

type Listener = Audience<u128, Vec<u128>>;
type Content = Msg<String, String>;

pub trait MessageHandler {
    fn start<T: Broadcast + Send + 'static>(self, caster: Arc<Mutex<T>>) -> JoinHandle<()>;
}

pub struct MessageReceiver(pub Receiver<(Listener, Content)>);

impl MessageHandler for MessageReceiver {
    fn start<T: Broadcast + Send + 'static>(self, caster: Arc<Mutex<T>>) -> JoinHandle<()> {
        spawn(move || {
            let caster = caster.clone();
            for (aud, msg) in self.0 {
                caster.lock().unwrap().send(&aud, &msg);
            }
        })
    }
}
