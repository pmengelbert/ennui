use crate::game::Game;
use crate::text::message::{Broadcast, Message, MessageFormat, Messenger};
use crate::text::Color::Green;
use crate::text::Wrap;
use crate::WriteResult;
use std::borrow::BorrowMut;
use std::io::Write;

impl<T> Broadcast for T
where
    T: BorrowMut<Game>,
{
    fn send(
        &mut self,
        audience: &dyn Messenger,
        message: &dyn Message,
    ) -> Vec<(u128, WriteResult)> {
        eprintln!("[{}]: made it to send func", "SUCCESS".color(Green));
eprintln!("in file {} on line number {}", file!(), line!());

        let g = self.borrow_mut();
        let mut v = vec![];

        let self_id = audience.id().unwrap_or_default();
        let object_id = audience.object().unwrap_or_default();
        let other_ids = audience.others();

        let self_msg = message.to_self();
        let obj_msg = message.to_object();
        let other_msg = message.to_others();
        eprintln!("this is the message going out to others: {:?}", other_msg);
eprintln!("in file {} on line number {}", file!(), line!());


        send_to_single_player(g, &mut v, self_id, self_msg);

        if let Some(msg) = obj_msg {
            let obj_msg = msg.wrap(90);
            send_to_single_player(g, &mut v, object_id, obj_msg);
        }

        if let Some(msg) = other_msg {
            let msg = msg.wrap(90);
            for id in other_ids {
                if let Some(p) = g.players.get_mut(&id) {
                    v.push((id, p.lock().unwrap().write(msg.as_bytes())));
                }
            }
        }

        v
    }
}

fn send_to_single_player(g: &mut Game, v: &mut Vec<(u128, WriteResult)>, id: u128, msg: String) {
    if let Some(p) = g.players.get_mut(&id) {
        let self_msg = msg.wrap(90);
        v.push((id, p.lock().unwrap().write(self_msg.as_bytes())));
    }
}
