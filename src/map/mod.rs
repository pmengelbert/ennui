use crate::game::{MapDir, Provider};
use crate::item::{ItemKind, ItemList};
use crate::player::{Player, PlayerIdList, PlayerList, Uuid};
use crate::text::Color::*;
use crate::text::Wrap;
use crate::PassFail;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

pub trait Locate {
    fn loc(&self) -> Coord;
    fn room<'a, T>(&self, rooms: &'a T) -> Option<&'a Room>
    where
        T: Provider<RoomList>,
    {
        let rooms: &RoomList = rooms.provide();
        rooms.get(&self.loc())
    }

    fn player_ids<'a, T>(&self, rooms: &'a T) -> Option<&'a PlayerIdList>
    where
        T: Provider<RoomList>,
    {
        Some(self.room(rooms)?.players())
    }

    fn players<'a, T>(&self, provider: &'a T) -> Vec<&'a Player>
    where
        T: Provider<RoomList> + Provider<PlayerList>,
    {
        let players: &PlayerList = provider.provide();
        let room = self.room(provider);
        room.unwrap_or(&Room::default())
            .players()
            .iter()
            .filter_map(|id| players.get(id))
            .collect()
    }

    fn player_by_name<'a, T, S>(&self, provider: &'a T, other: S) -> Option<&'a Player>
    where
        T: Provider<RoomList> + Provider<PlayerList>,
        S: AsRef<str>,
    {
        let pl: &PlayerList = provider.provide();
        pl.values()
            .find(|p| p.name() == other.as_ref() && p.loc() == self.loc())
    }

    fn player_by_name_mut<'a, T, S>(&self, provider: &'a mut T, other: S) -> Option<&'a mut Player>
    where
        T: Provider<RoomList> + Provider<PlayerList>,
        S: AsRef<str>,
    {
        let mut pl: &mut PlayerList = provider.provide_mut();
        pl.values_mut()
            .find(|p| p.name() == other.as_ref() && p.loc() == self.loc())
    }

    fn move_player<T, U>(&self, provider: &mut T, u: U, dir: MapDir) -> PassFail
    where
        T: Provider<RoomList> + Provider<PlayerList>,
        U: Uuid,
    {
        let u = u.uuid();
        let next_coord = self.loc().add(dir);
        {
            let rooms: &mut RoomList = provider.provide_mut();
            rooms.get_mut(&self.loc())?.players_mut().remove(&u);
            rooms.get_mut(&next_coord)?.players_mut().insert(u);
        }
        {
            let players: &mut PlayerList = provider.provide_mut();
            players.get_mut(&u)?.set_loc(next_coord);
        }

        Ok(())
    }
}

impl Provider<RoomList> for RoomList {
    fn provide(&self) -> &RoomList {
        self
    }

    fn provide_mut(&mut self) -> &mut RoomList {
        self
    }
}

impl<T> Locate for T
where
    T: AsRef<Coord>,
{
    fn loc(&self) -> Coord {
        *self.as_ref()
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Default, Clone, Copy)]
pub struct Coord(pub i64, pub i64);
impl AsRef<Coord> for Coord {
    fn as_ref(&self) -> &Coord {
        self
    }
}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct RoomList(HashMap<Coord, Room>);
impl Deref for RoomList {
    type Target = HashMap<Coord, Room>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RoomList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<RoomList> for RoomList {
    fn as_ref(&self) -> &RoomList {
        self
    }
}

#[derive(Debug, Default)]
pub struct Room {
    name: String,
    loc: Coord,
    description: String,
    players: PlayerIdList,
    items: ItemList,
}

impl AsMut<ItemList> for Room {
    fn as_mut(&mut self) -> &mut ItemList {
        self.items_mut()
    }
}

impl AsRef<ItemList> for Room {
    fn as_ref(&self) -> &ItemList {
        self.items()
    }
}

impl AsRef<Coord> for Room {
    fn as_ref(&self) -> &Coord {
        &self.loc
    }
}

impl Room {
    pub fn new(name: &str, description: Option<&str>, loc: Coord) -> Self {
        let name = name.to_owned();
        let description = description.unwrap_or("").to_owned();
        Self {
            name,
            description,
            loc: loc,
            players: PlayerIdList(HashSet::new()),
            items: ItemList::new(),
        }
    }

    pub fn display(&self, p: u128, global_players: &PlayerList) -> String {
        let Room {
            name,
            loc,
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

    pub fn items_mut(&mut self) -> &mut ItemList {
        &mut self.items
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

    #[test]
    fn locate() {
        assert_eq!(Coord(0, 0).loc(), Coord(0, 0));
    }
}
