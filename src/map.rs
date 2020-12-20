use std::collections::{HashSet};
use crate::player::{PlayerListRaw, Uuid};
use crate::item::{ItemList, ItemKind};

#[derive(Eq, PartialEq, Debug, Hash, Default, Clone, Copy)]
pub struct Coord(pub i64, pub i64);

#[derive(Debug, Default)]
pub struct Room {
    name: String,
    description: String,
    players: HashSet<u128>,
    items: ItemList,
}

impl Room {
    pub fn new(name: &str, description: Option<&str>) -> Self {
        let name = name.to_owned();
        let description = description.unwrap_or("").to_owned();
        Self {
            name,
            description,
            players: HashSet::new(),
            items: ItemList::new(),
        }
    }

    pub fn display(&self, global_players: &PlayerListRaw) -> String {
        let Room{ name, description, players, items } = self;
        let items_list = items.iter()
            .map(|i| {
                format!(" --> {}", i.name())
            })
            .collect::<Vec<_>>()
            .join("\n");
        let player_list = players.iter()
            .filter_map(|uuid| {
                match global_players.get(uuid) {
                    Some(player) => Some(format!("- {}", player.name())),
                    None => None,
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let underline = (0..self.name.len()).map(|_| '-').collect::<String>();
        format!("\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\
            ", name, underline, description, player_list, items_list)
    }

    pub fn players(&self) -> &HashSet<u128> {
        &self.players
    }

    pub fn players_mut(&mut self) -> &mut HashSet<u128> {
        &mut self.players
    }

    pub fn add_player<P>(&mut self, p: &P) -> bool
        where P: Uuid,
    {
        self.players.insert(p.uuid())
    }

    pub fn add_item(&mut self, i: ItemKind) {
        self.items.push(i)
    }

    pub fn get_item(&self, handle: &str) -> Option<&ItemKind> {
        self.items.get(handle)
    }

    pub fn items(&self) -> &ItemList {
        &self.items
    }

    pub fn get_itemlist(&mut self) -> ItemList {
        std::mem::replace(&mut self.items, ItemList::new())
    }

    pub fn replace_itemlist(&mut self, i: ItemList) {
        self.items = i;
    }
}

#[cfg(test)]
mod room_test {
    use super::*;
    use crate::player::Player;

    #[test]
    fn room_display_sample() {
        use crate::player::PlayerList;
        let mut pl = PlayerList::new();
        let p = Player::new("bill");
        let q = Player::new("mindy");
        let mut r = Room {
            name: "the room".to_owned(),
            description: "this is your room".to_owned(),
            players: HashSet::new(),
            items: ItemList::new(),
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

// pub struct Map {
//     m: HashMap<Coord, Room>
// }

#[cfg(test)]
mod map_test {
    use super::*;

    #[test]
    fn map_test() {
        assert_eq!(Coord(0, 0).north(), Coord(0, 1));
    }
}


