use super::Error;
use crate::game::Game;
use crate::item::error::Error::{ItemNotFound, TooHeavy};
use crate::item::ItemKind::Scenery;
use crate::item::{Holder, ItemKind, ItemTrait};
use crate::map::coord::Coord;
use crate::map::RoomList;
use crate::player::{PlayerList, Uuid};
use std::sync::Arc;

#[derive(Clone)]
pub enum Direction {
    Take,
    Give,
    Drop,
    Wear,
    Remove,
}

impl Game {
    pub fn transfer<T>(
        &mut self,
        u: T,
        other: Option<&str>,
        dir: Direction,
        handle: &str,
    ) -> Result<String, Error>
    where
        T: Uuid,
    {
        use Direction::*;

        let uuid = u.uuid();
        let loc = &self.loc_of(u).unwrap_or_default();
        let oid = self.id_of(other.unwrap_or_default());
        let other_id = oid.unwrap_or_default();

        self.validate_other_player(other, loc, dir.clone())?;

        let rooms = &mut self.rooms;
        let players = &mut self.players;

        match dir {
            Take => Self::take(rooms, players, uuid, loc, handle),
            Drop => Self::drop(rooms, players, uuid, loc, handle),
            Give => Self::give(players, (uuid, other_id), other, handle),
            Wear => Self::wear(players, uuid, handle),
            Remove => Self::remove(players, uuid, handle),
        }
    }

    fn take(
        rooms: &mut RoomList,
        players: &mut PlayerList,
        uuid: u128,
        loc: &Coord,
        handle: &str,
    ) -> Result<String, Error> {
        let room = match rooms.get_mut(loc) {
            Some(r) => r,
            None => return Err(a(handle)),
        };

        if let Some(Scenery(_)) = room.items().get(handle) {
            return Err(Arc::new(TooHeavy(handle.to_owned())));
        }

        let player = match players.get_mut(&uuid) {
            Some(p) => p,
            None => return Err(a(handle)),
        };
        room.transfer(player, handle).map_err(|e| a(&e))
    }

    fn drop(
        rooms: &mut RoomList,
        players: &mut PlayerList,
        uuid: u128,
        loc: &Coord,
        handle: &str,
    ) -> Result<String, Error> {
        let room = match rooms.get_mut(loc) {
            Some(r) => r,
            None => return Err(a(handle)),
        };
        let player = match players.get_mut(&uuid) {
            Some(p) => p,
            None => return Err(a(handle)),
        };
        player.transfer(room, handle).map_err(|e| a(&e))
    }

    fn give(
        players: &mut PlayerList,
        ids: (u128, u128),
        other_name: Option<&str>,
        handle: &str,
    ) -> Result<String, Error> {
        use crate::item::error::Error::PlayerNotFound;

        let (uuid, other_id) = ids;
        let item = {
            let p = match players.get_mut(&uuid) {
                Some(p) => p,
                None => return Err(a(handle)),
            };
            match p.remove_item(handle) {
                Some(i) => i,
                None => return Err(a(handle)),
            }
        };

        let item_name = item.name().to_owned();
        let other_p = match players.get_mut(&other_id) {
            Some(p) => p,
            None => {
                return Err(Arc::new(PlayerNotFound(
                    other_name.unwrap_or_default().to_owned(),
                )))
            }
        };

        other_p.give_item(item);
        Ok(item_name)
    }

    fn wear(players: &mut PlayerList, uuid: u128, handle: &str) -> Result<String, Error> {
        use crate::item::error::Error::Clothing;
        let (items, clothing) = {
            match players.get_mut(&uuid) {
                Some(p) => p,
                None => return Err(a(handle)),
            }
            .all_items_mut()
        };
        match items.get(handle) {
            Some(ItemKind::Clothing(_)) => (),
            None => return Err(a(handle)),
            _ => return Err(Arc::new(Clothing(handle.to_owned()))),
        }
        items.transfer(clothing, handle).map_err(|e| a(&e))
    }

    fn remove(players: &mut PlayerList, uuid: u128, handle: &str) -> Result<String, Error> {
        let (items, clothing) = {
            match players.get_mut(&uuid) {
                Some(p) => p,
                None => return Err(a(handle)),
            }
            .all_items_mut()
        };
        clothing.transfer(items, handle).map_err(|e| a(&e))
    }

    fn validate_other_player(
        &self,
        other: Option<&str>,
        loc: &Coord,
        dir: Direction,
    ) -> Result<(), Error> {
        use crate::item::error::Error::PlayerNotFound;
        use Direction::*;

        let oid = self.id_of(other.unwrap_or_default());
        let other_id = oid.unwrap_or_default();

        if let Give = dir.clone() {
            match (other, oid) {
                (Some(_), None) => {
                    return Err(Arc::new(PlayerNotFound(
                        other.unwrap_or_default().to_owned(),
                    )))
                }
                _ => (),
            }

            match self.loc_of(other_id) {
                Some(other_loc) if &other_loc != loc => {
                    return Err(Arc::new(PlayerNotFound(
                        other.unwrap_or_default().to_owned(),
                    )))
                }
                _ => (),
            }
        }

        Ok(())
    }
}

fn a<T: AsRef<str>>(s: T) -> Error {
    let s = s.as_ref().to_owned();
    Arc::new(ItemNotFound(s))
}
