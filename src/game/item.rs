use std::option::NoneError;
use std::sync::Arc;

use crate::game::Game;
use crate::item::error::Error::{FatalError, Guarded, ItemNotFound, PlayerNotFound, TooHeavy};
use crate::item::Item::Scenery;
use crate::item::{Attribute, Describe, Item, Quality};
use crate::map::coord::Coord;
use crate::map::RoomList;
use crate::player::list::PlayerList;
use crate::player::Uuid;
use crate::text::article;

use super::Error;
use crate::item::list::{Holder, ItemListTrait};

#[derive(Clone, Copy)]
pub enum Direction {
    Take,
    Give,
    Drop,
    Wear,
    Remove,
}

pub enum TransferResult {
    GuardAppeased(String),
    Ok(String),
    Err(Error),
}

impl From<Result<String, Error>> for TransferResult {
    fn from(t: Result<String, Error>) -> Self {
        match t {
            Ok(s) => TransferResult::Ok(s),
            Err(s) => TransferResult::Err(s),
        }
    }
}

impl From<NoneError> for TransferResult {
    fn from(_: NoneError) -> Self {
        TransferResult::Err(Arc::new(crate::item::error::Error::ItemNotFound(
            "WHAT. ARE. YOU. TALKING. ABOUT.".to_string(),
        )))
    }
}

impl Game {
    pub fn transfer<T>(
        &mut self,
        u: T,
        other: Option<&str>,
        dir: Direction,
        handle: &str,
    ) -> TransferResult
    where
        T: Uuid,
    {
        use Direction::*;

        let uuid = u.uuid();
        let loc = &self.loc_of(u).unwrap_or_default();
        let oid = self.id_of(other.unwrap_or_default());
        let other_id = oid.unwrap_or_default();

        if let Err(_) = self.validate_other_player(other, loc, dir) {
            return TransferResult::Err(Arc::new(PlayerNotFound(
                other.unwrap_or_default().to_owned(),
            )));
        };

        let rooms = &mut self.rooms;
        let players = &mut self.players;

        match dir {
            Take => Self::take(rooms, players, uuid, loc, handle).into(),
            Drop => Self::drop(rooms, players, uuid, loc, handle).into(),
            Give => Self::give(players, rooms, loc, (uuid, other_id), other, handle),
            Wear => Self::wear(players, uuid, handle).into(),
            Remove => Self::remove(players, uuid, handle).into(),
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
        room.transfer(player, handle.into()).map_err(|e| a(&e))
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
        rooms: &mut RoomList,
        loc: &Coord,
        ids: (u128, u128),
        other_name: Option<&str>,
        handle: &str,
    ) -> TransferResult {
        use TransferResult::*;

        let (uuid, other_id) = ids;
        let item = {
            let p = match players.get_mut(&uuid) {
                Some(p) => p,
                None => return Err(a(handle)),
            };

            match p.items_mut().get_owned(handle) {
                Some(i) => i,
                None => return Err(a(handle)),
            }
        };

        let item_name = item.name().to_owned();
        let other_p = match players.get_mut(&other_id) {
            Some(p) => p,
            None => {
                let room = match rooms.get_mut(&loc) {
                    Some(r) => r,
                    None => {
                        return Err(Arc::new(PlayerNotFound(
                            other_name.unwrap_or_default().to_owned(),
                        )));
                    }
                };
                match room.get_mut(other_name.unwrap_or_default()) {
                    Some(Item::Guard(_, guard)) => {
                        use std::result::Result::*;
                        match guard.insert(item) {
                            Ok(()) => {
                                return GuardAppeased(format!(
                                    "you see {} relax a little bit. maybe now they'll let you through",
                                    article(guard.name())
                                ));
                            }
                            Err(given_back) => {
                                match players.get_mut(&uuid) {
                                    Some(p) => {
                                        if p.insert(given_back).is_err() {
                                            return Err(Arc::new(
                                                crate::item::error::Error::FatalError(
                                                    "wasn't able to return item to player after \
                                                    failed transfer to guard type."
                                                        .into(),
                                                ),
                                            ))
                                            .into();
                                        }
                                    }
                                    None => (),
                                }
                                return TransferResult::Err(Arc::new(Guarded(
                                    guard.name().to_owned(),
                                )));
                            }
                        };
                    }
                    _ => {
                        return Err(Arc::new(PlayerNotFound(
                            other_name.unwrap_or_default().to_owned(),
                        )));
                    }
                }
            }
        };

        if other_p.items_mut().insert(item).is_err() {
            return Err(Arc::new(FatalError("COULD NOT TRANSFER ITEM".into())));
        };
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
            Some(i) if i.is(Quality::Clothing) => (),
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
        use Direction::*;

        let oid = self.id_of(other.unwrap_or_default());
        let other_id = oid.unwrap_or_default();

        let rooms = &self.rooms;

        if let Give = dir {
            println!("checkpoint 1");
            match (other, oid) {
                (Some(_), None) => {
                    println!("checkpoint 2");
                    match rooms.get(loc) {
                        Some(room) => {
                            println!("checkpoint 3");
                            match room.get(other.unwrap_or_default()) {
                                Some(_) => {
                                    println!("checkpoint 4");
                                    return Ok(());
                                }
                                None => (),
                            }
                        }
                        None => (),
                    }
                    return Err(Arc::new(PlayerNotFound(
                        other.unwrap_or_default().to_owned(),
                    )));
                }
                _ => (),
            }

            match self.loc_of(other_id) {
                Some(other_loc) if &other_loc != loc => {
                    println!("made it!");
                    return Err(Arc::new(PlayerNotFound(
                        other.unwrap_or_default().to_owned(),
                    )));
                }
                None => {
                    println!("made it!");
                    return Ok(());
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
