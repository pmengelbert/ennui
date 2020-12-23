use crate::game::MapDir;
use crate::item::{ItemKind, ItemList};
use crate::player::{PlayerIdList, PlayerList, Uuid};
use crate::text::Color::*;
use crate::text::Wrap;
use std::collections::HashSet;

#[derive(Eq, PartialEq, Debug, Hash, Default, Clone, Copy)]
pub struct Coord(pub i64, pub i64);

#[derive(Debug, Default)]
pub struct Room {
    name: String,
    description: String,
    players: PlayerIdList,
    items: ItemList,
}

impl Room {
    pub fn new(name: &str, description: Option<&str>) -> Self {
        let name = name.to_owned();
        let description = description.unwrap_or("").to_owned();
        Self {
            name,
            description,
            players: PlayerIdList(HashSet::new()),
            items: ItemList::new(),
        }
    }

    pub fn display(&self, p: u128, global_players: &PlayerList) -> String {
        let Room {
            name,
            description,
            players,
            items,
        } = self;

        let player_list = players
            .iter()
            .filter_map(|uuid| match global_players.get(uuid) {
                Some(player) if player.uuid() != p && player.uuid() != 0 => Some(player.name()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let player_list = Yellow(match player_list.len() {
            0 => "".to_owned(),
            1 => format!("\n{}", player_list[0]),
            _ => format!("\n{}", player_list.join("\n")),
        });

        let items_list = items.iter().map(|i| i.name()).collect::<Vec<_>>();
        let items_list = Green(match items_list.len() {
            0 => "".to_owned(),
            1 => format!("\n{}", items_list[0]),
            _ => format!("\n{}", items_list.join("\n")),
        });

        let underline = (0..self.name.len()).map(|_| '-').collect::<String>();

        format!(
            "{}\n\
            {}\n\
            {}\
            {}\
            {}",
            name,
            underline,
            description.wrap(80),
            player_list,
            items_list,
        )
    }

    pub fn players(&self) -> &PlayerIdList {
        &self.players
    }

    pub fn players_mut(&mut self) -> &mut PlayerIdList {
        &mut self.players
    }

    pub fn add_player<P>(&mut self, p: &P) -> bool
    where
        P: Uuid,
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
            players: PlayerIdList(HashSet::new()),
            items: ItemList::new(),
        };
        r.players.insert(p.uuid());
        r.players.insert(q.uuid());
        pl.insert(p.uuid(), p);
        pl.insert(q.uuid(), q);
    }
}

impl Coord {
    pub fn north(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x, *y + 1)
    }

    pub fn south(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x, *y - 1)
    }

    pub fn east(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x + 1, *y)
    }

    pub fn west(&self) -> Self {
        let Coord(x, y) = self;
        Coord(*x - 1, *y)
    }

    pub fn add(&self, dir: MapDir) -> Self {
        use MapDir::*;

        match dir {
            North => self.north(),
            South => self.south(),
            East => self.east(),
            West => self.west(),
            _ => return *self,
        }
    }

    pub fn north_mut(&mut self) {
        let Coord(x, y) = self;
        *y += 1;
    }

    pub fn south_mut(&mut self) {
        let Coord(x, y) = self;
        *y -= 1;
    }

    pub fn east_mut(&mut self) {
        let Coord(x, y) = self;
        *x += 1;
    }

    pub fn west_mut(&mut self) {
        let Coord(x, y) = self;
        *x -= 1;
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

#[cfg(test)]
mod map_test {
    use super::*;

    #[test]
    fn map_test() {
        assert_eq!(Coord(0, 0).north(), Coord(0, 1));
    }
}
