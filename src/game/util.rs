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
