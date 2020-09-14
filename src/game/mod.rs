use super::player::Player;

pub struct Game {
    x: i32,
}

impl Game {
    pub fn new() -> Self {
        Game {
            x: 6,
        }
    }
}
