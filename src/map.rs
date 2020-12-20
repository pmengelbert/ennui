use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::player::{PlayerList, Player, PlayerListRaw, UuidProvide};

#[derive(Eq, PartialEq, Debug, Hash)]
pub struct Coord(pub i64, pub i64);

#[derive(Debug)]
pub struct Room {
    name: String,
    description: String,
    players: HashSet<u128>,
}

impl Room {
    pub fn new(name: &str, description: Option<&str>) -> Self {
        let name = name.to_owned();
        let description = description.unwrap_or("").to_owned();
        Self {
            name,
            description,
            players: HashSet::new(),
        }
    }

    pub fn display(&self, globalPlayers: &PlayerListRaw) -> String {
        let Room{name, description, players} = self;
        let player_list = players.iter()
            .filter_map(|uuid| {
                match globalPlayers.get(uuid) {
                    Some(player) => Some(format!("- {}", player.name())),
                    None => None,
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let underline = (0..self.name.len()).map(|i| '-').collect::<String>();
        format!("\n\
            {}\n\
            {}\n\
            {}\n\
            {}\
            ", name, underline, description, player_list)
    }

    pub fn add_player<P>(&mut self, p: &P) -> bool
        where P: UuidProvide,
    {
        self.players.insert(p.uuid())
    }
}

#[cfg(test)]
mod room_test {
    use super::*;
    use crate::player::Player;

    #[test]
    fn room_display_sample() {
        let mut pl = PlayerList::new();
        let p = Player::new("bill");
        let q = Player::new("mindy");
        let mut r = Room {
            name: "the room".to_owned(),
            description: "this is your room".to_owned(),
            players: HashSet::new(),
        };
        r.players.insert(p.uuid());
        r.players.insert(q.uuid());
        pl.insert(p.uuid(), p);
        pl.insert(q.uuid(), q);
        println!("{}", r.display(&pl));
    }
}

impl Coord {
    pub fn north(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x, *y+1)
    }

    pub fn south(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x, *y-1)
    }

    pub fn east(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x+1, *y)
    }

    pub fn west(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x-1, *y)
    }
}

#[cfg(test)]
mod coord_test {
    use super::*;

    #[test]
    fn coord_test_north() {
        assert_eq!(Coord(0, 0).north(), Coord(0, 1));
    }

    #[test]
    fn coord_test_south() {
        assert_eq!(Coord(0, 0).south(), Coord(0, -1));
    }

    #[test]
    fn coord_test_east() {
        assert_eq!(Coord(0, 0).east(), Coord(1, 0));
    }

    #[test]
    fn coord_test_west() {
        assert_eq!(Coord(0, 0).west(), Coord(-1, 0));
    }
}

pub struct Map {
    m: HashMap<Coord, Room>
}

#[cfg(test)]
mod map_test {
    use super::*;

    #[test]
    fn map_test() {
        assert_eq!(Coord(0, 0).north(), Coord(0, 1));
    }
}


