use std::borrow::Cow;
use std::io;

use crate::player::{PlayerType, Uuid};
use crate::text::Color;
use std::sync::{Arc, Mutex};

type WriteResult = io::Result<usize>;

pub trait Message: Send + Sync {
    fn to_self(&self) -> String;
    fn to_object(&self) -> Option<String>;
    fn to_others(&self) -> Option<String>;
}

impl<T, U> Message for Msg<T, U>
where
    T: AsRef<str> + Send + Sync,
    U: AsRef<str> + Send + Sync,
{
    fn to_self(&self) -> String {
        self.s.as_ref().to_owned()
    }

    fn to_object(&self) -> Option<String> {
        None
    }

    fn to_others(&self) -> Option<String> {
        Some(self.o.as_ref()?.as_ref().to_owned())
    }
}

pub trait Messenger: Send + Sync {
    fn id(&self) -> Option<u128>;

    fn object(&self) -> Option<u128> {
        None
    }

    fn others(&self) -> Vec<u128> {
        vec![]
    }
}

impl Messenger for u128 {
    fn id(&self) -> Option<u128> {
        Some(*self)
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
    fn send(&mut self, audience: &dyn Messenger, message: &dyn Message)
        -> Vec<(u128, WriteResult)>;
}

pub struct Audience<T, U>(pub T, pub U)
where
    T: Uuid + Send + Sync,
    U: Uuid + Send + Sync;

#[derive(Clone)]
pub struct FightAudience(pub u128, pub u128, pub Vec<u128>);
impl Messenger for FightAudience {
    fn id(&self) -> Option<u128> {
        Some(self.0)
    }

    fn object(&self) -> Option<u128> {
        Some(self.1)
    }

    fn others(&self) -> Vec<u128> {
        self.2.clone()
    }
}

impl<T, U> Messenger for Audience<T, U>
where
    T: Uuid + Send + Sync,
    U: Uuid + Send + Sync,
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
        self.1.others()
    }
}

#[derive(Eq, PartialEq)]
pub struct Msg<T, U>
where
    T: AsRef<str> + Send + Sync,
    U: AsRef<str> + Send + Sync,
{
    pub s: T,
    pub o: Option<U>,
}

impl Message for &str {
    fn to_self(&self) -> String {
        self.to_string()
    }

    fn to_object(&self) -> Option<String> {
        Some(self.to_string())
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

    fn to_object(&self) -> Option<String> {
        let mut s = String::new();
        s.push_str(&self);
        Some(s)
    }

    fn to_others(&self) -> Option<String> {
        let mut s = String::new();
        s.push_str(&self);
        Some(s)
    }
}

impl Message for String {
    fn to_self(&self) -> String {
        self.clone()
    }

    fn to_object(&self) -> Option<String> {
        Some(self.clone())
    }

    fn to_others(&self) -> Option<String> {
        Some(self.clone())
    }
}

#[cfg(test)]
mod test_message {
    use crate::game::Game;
    use crate::location::Coord;
    use crate::map::Space;

    use super::*;
    use crate::player::list::PlayerIdListTrait;

    #[test]
    fn test_message_1() {
        let mut g = Game::new().unwrap();
        let s = "poo butts poo";
        let n = 8_u128;
        g.send(&n, &s);
    }

    #[test]
    fn test_message_2() {
        let mut g = Game::new().unwrap();
        let s = "poo butts poo".to_owned();
        let n = 8_u128;
        g.send(&n, &s);
    }

    #[test]
    fn test_message_3() {
        let mut g = Game::new().unwrap();
        let s = "poo butts poo".to_owned();
        let n = 8_u128;
        let room = g.get_room(Coord(0, 0)).unwrap();
        let _audience = Audience(n, room.players().except(n));
        g.send(&n, &s);
    }
}

pub trait MessageFormat {
    fn un_padded(&self) -> String;
    fn padded(&self) -> String {
        let mut b = String::new();
        b.push('\n');
        b.push_str(&self.un_padded());
        b.push_str("\n\n > ");
        b
    }

    fn custom_padded(&self, before: &str, after: &str) -> String {
        let mut s = String::new();
        s.push_str(before);
        s.push_str(&self.un_padded());
        s.push_str(after);
        s
    }

    fn color(&self, color: Color) -> String {
        let mut s = String::new();
        s.push('\u{001b}');
        s.push_str(color.to_code());
        s.push_str(&self.un_padded());
        s.push_str("\u{001b}[37m");
        s
    }
}

impl MessageFormat for String {
    fn un_padded(&self) -> String {
        self.clone()
    }

    #[cfg(target_arch = "wasm32")]
    fn padded(&self) -> String {
        wasm_padded(self)
    }
}

impl MessageFormat for &str {
    fn un_padded(&self) -> String {
        (*self).to_owned()
    }

    #[cfg(target_arch = "wasm32")]
    fn padded(&self) -> String {
        wasm_padded(self)
    }
}

impl Messenger for Arc<Mutex<PlayerType>> {
    fn id(&self) -> Option<u128> {
        Some(self.lock().unwrap().uuid())
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_padded(s: &str) -> String {
    let mut b = String::new();
    b.push('\n');
    b.push_str(&s.un_padded());
    b.push_str("\n");
    b
}
