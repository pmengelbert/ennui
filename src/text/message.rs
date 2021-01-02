use crate::player::Uuid;
use std::io;
use std::borrow::Cow;



type WriteResult = io::Result<usize>;

pub trait Message {
    fn to_self(&self) -> String;
    fn to_others(&self) -> Option<String>;
}

impl<T, U> Message for Msg<T, U>
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    fn to_self(&self) -> String {
        self.s.as_ref().to_owned()
    }

    fn to_others(&self) -> Option<String> {
        Some(self.o.as_ref()?.as_ref().to_owned())
    }
}

pub trait Messenger {
    fn id(&self) -> Option<u128>;
    fn others(&self) -> Vec<u128> {
        vec![]
    }
}

impl Uuid for Vec<u128> {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Vec<u128> {
        self.clone()
    }
}

pub trait Broadcast {
    fn send<A, M>(&mut self, audience: A, message: M) -> Vec<WriteResult>
    where
        A: Messenger,
        M: Message;
}

impl Messenger for u128 {
    fn id(&self) -> Option<u128> {
        Some(*self)
    }
}

pub struct Audience<T, U>(pub T, pub U)
where
    T: Uuid,
    U: Uuid;

impl<T, U> Messenger for Audience<T, U>
where
    T: Uuid,
    U: Uuid,
{
    fn id(&self) -> Option<u128> {
        let u = self.0.uuid();
        if u == 0 {
            None
        } else {
            Some(u)
        }
    }

    fn others(&self) -> Vec<u128> {
        let v: Vec<u128> = self
            .1
            .others()
            .iter()
            .cloned()
            .collect();
        v
    }
}

#[derive(Eq, PartialEq)]
pub struct Msg<T, U>
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    pub s: T,
    pub o: Option<U>,
}

impl Message for &str {
    fn to_self(&self) -> String {
        self.to_string()
    }

    fn to_others(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl Message for Cow<'static, str> {
    fn to_self(&self) -> String {
        let mut s = String::new();
        s.push_str(&self);
        s
    }

    fn to_others(&self) -> Option<String> {
        let mut s = String::new();
        s.push_str(&self);
        Some(s)
    }
}

impl Message for &String {
    fn to_self(&self) -> String {
        (*self).clone()
    }

    fn to_others(&self) -> Option<String> {
        Some((*self).clone())
    }
}

#[cfg(test)]
mod test_message {
    use super::*;
    use crate::game::Game;
    use crate::map::coord::Coord;
    use crate::map::Space;

    #[test]
    fn test_message_1() {
        let mut g = Game::new().unwrap();
        let s = "poo butts poo";
        let n = 8_u128;
        g.send(n, s);
    }

    #[test]
    fn test_message_2() {
        let mut g = Game::new().unwrap();
        let s = "poo butts poo".to_owned();
        let n = 8_u128;
        g.send(n, &s);
    }

    #[test]
    fn test_message_3() {
        let mut g = Game::new().unwrap();
        let s = "poo butts poo".to_owned();
        let n = 8_u128;
        let room = g.get_room(&Coord(0, 0)).unwrap();
        let _audience = Audience(n, room.players().except(n));
        g.send(n, &s);
    }
}
