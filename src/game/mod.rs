use super::player::{UUID, Player, PlayerType, PlayerType::*};
use super::map::{Map};
use std::collections::HashMap;

pub struct Game {
    players: HashMap<UUID, Player>,
    map: Map,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new_test();
        Game {
            players: HashMap::new(),
            map: map,
        }
    }

    pub fn add_player(&mut self, p: Player) -> Option<Player> {
        let uuid = p.uuid();

        self.players.insert(uuid, p)
    }
}
