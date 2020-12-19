use ennui::player::Player;
use std::collections::HashMap;

pub struct Game {
    map: HashMap<String, Player>,
}

#[cfg(test)]
mod butts {
    use super::*;

    #[test]
    fn poo() {
        assert_eq!(1, 1);
    }
}