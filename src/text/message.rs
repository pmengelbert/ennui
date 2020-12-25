use crate::player::Uuid;
use std::io;

type WriteResult = io::Result<usize>;

pub trait Message {
    fn to_self(&self) -> String;
    fn to_others(&self) -> Option<String>;
}

impl<T> Message for T
where
    T: AsRef<str>,
{
    fn to_self(&self) -> String {
        self.as_ref().to_owned()
    }

    fn to_others(&self) -> Option<String> {
        None
    }
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
    fn others(&self) -> Option<Vec<u128>> {
        None
    }
}

impl Uuid for Option<Vec<u128>> {
    fn uuid(&self) -> u128 {
        0
    }

    fn others(&self) -> Option<Vec<u128>> {
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

    fn others(&self) -> Option<Vec<u128>> {
        let id = self.id();
        let v: Vec<u128> = self
            .1
            .others()?
            .iter()
            .cloned()
            .filter(|&i| Some(i) != id)
            .collect();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
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
