use super::map::{room::Room, Coord, Map};
use super::player::{Player, PlayerType, PlayerType::*, UUID};
use crate::item::{Item, ItemType, ItemType::*};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct Game {
    players: HashMap<UUID, PlayerType<Player>>,
    npcs: HashMap<UUID, PlayerType<Player>>,
    admins: HashMap<UUID, PlayerType<Player>>,
    map: Map,
}

pub enum Direction {
    To,
    From,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new_test();

        Game {
            players: HashMap::new(),
            npcs: HashMap::new(),
            admins: HashMap::new(),
            map: map,
        }
    }

    pub fn add_player(&mut self, p: PlayerType<Player>) -> Option<PlayerType<Player>> {
        let insertion_point = &mut self.players;

        let player = p.player();

        insertion_point.insert(player.uuid(), p)
    }

    pub fn place_player_in_room(&mut self, uuid: UUID, c: Coord) -> Result<String, String> {
        let mut player = match self.players.get_mut(&uuid) {
            Some(q) => q.player_mut(),
            None => {
                return Err(format!("somehow, player not found"));
            }
        };
        let old_coord = player.location();

        if let Some(old_room) = self.map.get_mut(old_coord) {
            old_room.remove_player(uuid);
        }

        match self.map.get_mut(c) {
            Some(r) => {
                player.set_location(c);
                r.add_player(uuid);
                Ok(self.room_to_string_for_player(uuid))
            }
            None => Err("room does not exist".to_string()),
        }
    }

    pub fn room_to_string_for_player(&self, uuid: UUID) -> String {
        let map = &self.map;

        match self.players.get(&uuid) {
            Some(o) => {
                let p = o.player();

                match map.get(p.location()) {
                    Some(room) => {
                        let mut s = String::new();
                        let (name, items) = room.to_string();
                        s.push_str(&name);

                        for u in room.players.iter().filter(|&u| *u != uuid) {
                            if let Some((name, _, items)) = self.get_player_info_strings(*u) {
                                s.push_str(&format!("\n ---> {}", name))
                            }
                        }

                        s.push_str(&items);
                        s
                    }
                    None => "".to_string(),
                }
            }
            None => "".to_string(),
        }
    }

    pub fn look_at_item(&self, uuid: UUID, item_hook: &str) -> String {
        let (p, location) = {
            let p = self.get_player(uuid);
            (p, p.location())
        };

        match self.map.get(location) {
            Some(room) => {
                match room.players.iter().find(|&&u| {
                    let p = self.get_player(u);
                    p.name() == item_hook
                }) {
                    Some(u) => self.get_player(*u).description().clone(),
                    None => match room.items_not_mut().find_by_hook(item_hook) {
                        Some(i) => i.description().clone(),
                        None => match p.hands().find_by_hook(item_hook) {
                            Some(i) => i.description().clone(),
                            None => format!("you don't see a {} here", item_hook),
                        },
                    },
                }
            }
            None => {
                return format!("unable to find room");
            }
        }
    }

    pub fn get_player_info_strings(&self, uuid: UUID) -> Option<(String, String, String)> {
        let mut s = String::new();
        match self.players.get(&uuid) {
            Some(o) => {
                let p = o.player();
                s.push_str(&p.hands().to_string());
                Some((p.name().clone(), p.description().clone(), s))
            }
            None => None,
        }
    }

    pub fn get_player(&self, uuid: UUID) -> &Player {
        self.players.get(&uuid).unwrap().player()
    }

    pub fn get_player_mut(&mut self, uuid: UUID) -> &mut Player {
        match self.players.get_mut(&uuid).unwrap() {
            Human(ref mut p) | NPC(ref mut p) | Admin(ref mut p) => p,
        }
    }

    pub fn get_room(&self, c: Coord) -> Option<&Room> {
        self.map.get(c)
    }

    pub fn get_room_mut(&mut self, c: Coord) -> Option<&mut Room> {
        self.map.get_mut(c)
    }

    pub fn player_takes_item(
        &mut self,
        uuid: UUID,
        item_hook: &str,
        dir: Direction,
    ) -> Result<String, String> {
        let map = &mut self.map;
        let players = &mut self.players;

        let (c, hands) = match players.get_mut(&uuid) {
            Some(p) => {
                let p = p.player_mut();
                let c = p.location();
                let hands = p.hands_mut();
                (c, hands)
            }
            None => {
                return Err(format!("unable to find player"));
            }
        };

        let room = match map.get_mut(c) {
            Some(rm) => rm.items(),
            None => {
                return Err(format!("unable to find room"));
            }
        };

        let (to, from, verb) = match dir {
            Direction::To => (hands, room, "drop"),
            Direction::From => (room, hands, "take"),
        };

        match to.transfer_item(item_hook, from) {
            Ok(_) => Ok(format!("you {} the {}", verb, item_hook)),
            e => e,
        }
    }

    pub fn player_wears_item(
        &mut self,
        uuid: UUID,
        item_hook: &str,
        dir: Direction,
    ) -> Result<String, String> {
        let player = match self.players.entry(uuid) {
            Entry::Occupied(oe) => oe.into_mut(),
            _ => {
                return Err(format!("can't find player"));
            }
        };

        let (mut hands, mut worn) = {
            match player {
                Human(p) => (&mut p.hands, &mut p.worn),
                _ => {
                    return Err(format!("nonhuman"));
                }
            }
        };

        let (to, from, verb) = match dir {
            Direction::To => (&mut hands, &mut worn, "wear"),
            Direction::From => (&mut worn, &mut hands, "remove"),
        };

        match to.transfer_item(item_hook, from) {
            Ok(_) => Ok(format!("you {} the {}", verb, item_hook)),
            e => e,
        }
    }

    pub fn list_items_for_player(&mut self, uuid: UUID) -> String {
        let p = self.get_player(uuid);
        let mut s = String::from("you are carrying:");

        if let Some(c) = p.hands().container() {
            for i in c {
                s.push_str(&format!("\n - {}", i.item().name()));
            }
        }

        s
    }
}
