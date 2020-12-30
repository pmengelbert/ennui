use super::*;

impl AsRef<Game> for Game {
    fn as_ref(&self) -> &Game {
        self
    }
}

impl AsMut<Game> for Game {
    fn as_mut(&mut self) -> &mut Game {
        self
    }
}

impl AsRef<RoomList> for Game {
    fn as_ref(&self) -> &RoomList {
        &self.rooms
    }
}

impl AsRef<PlayerList> for Game {
    fn as_ref(&self) -> &PlayerList {
        &self.players
    }
}

impl AsMut<RoomList> for Game {
    fn as_mut(&mut self) -> &mut RoomList {
        &mut self.rooms
    }
}

impl AsMut<PlayerList> for Game {
    fn as_mut(&mut self) -> &mut PlayerList {
        &mut self.players
    }
}

pub fn random_insult() -> String {
    match rand::thread_rng().gen_range(1, 6) {
        1 => "dude wtf",
        2 => "i think you should leave",
        3 => "i'll have to ask my lawyer about that",
        4 => "that's ... uncommon",
        _ => "that's an interesting theory... but will it hold up in the laboratory?",
    }
    .to_owned()
}

pub fn to_buf<T: AsRef<str>>(msg: T) -> Vec<u8> {
    let buf = msg.as_ref().as_bytes();
    let mut b = vec![];
    b.extend_from_slice(b"\n".as_ref());
    b.extend_from_slice(buf.as_ref());
    b.extend_from_slice(b"\n\n > ".as_ref());
    b
}
