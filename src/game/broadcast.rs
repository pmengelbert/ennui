use crate::game::util::to_buf;
use crate::game::Game;
use crate::text::message::{Broadcast, Message, Messenger};
use crate::text::Wrap;
use crate::WriteResult;
use std::borrow::BorrowMut;
use std::io::Write;

impl<T> Broadcast for T
where
    T: BorrowMut<Game>,
{
    fn send<A, M>(&mut self, audience: A, message: M) -> Vec<WriteResult>
    where
        A: Messenger,
        M: Message,
    {
        let g = self.borrow_mut();
        let mut v = vec![];
        let self_id = audience.id().unwrap_or_default();
        let other_ids = audience.others().unwrap_or_default();

        let self_msg = message.to_self();
        let other_msg = message.to_others();

        if let Some(p) = g.players.get_mut(&self_id) {
            let self_msg = self_msg.wrap(90);
            v.push(p.write(to_buf(self_msg).as_slice()));
        }

        if let Some(msg) = other_msg {
            let msg = msg.wrap(90);
            for id in other_ids {
                if let Some(p) = g.players.get_mut(&id) {
                    v.push(p.write(to_buf(&msg).as_slice()));
                }
            }
        }

        v
    }
}