use std::option::NoneError;
use std::sync::Arc;

use crate::game::Game;
use crate::item::error::Error::{PlayerNotFound};
use crate::item::Item::Scenery;
use crate::item::{Attribute, Describe, Item, Quality};
use crate::map::coord::Coord;
use crate::map::list::RoomList;
use crate::player::list::PlayerList;
use crate::player::{Player, Uuid};
use crate::text::article;

use super::Error;
use crate::error::EnnuiError::{Fatal, Message, Simple};
use crate::error::CmdErr::{ItemNotFound, NotClothing, TooHeavy};
use crate::error::{EnnuiError};
use crate::item::list::{Holder, ListTrait};

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
    ) -> Result<String, EnnuiError>
    where
        T: Uuid,
    {
        use Direction::*;

        let uuid = u.uuid();
        let loc = &self.loc_of(u).unwrap_or_default();
        let oid = self.id_of(other.unwrap_or_default());
        let other_id = oid.unwrap_or_default();

        if let Err(_) = self.validate_other_player(other, loc, dir) {
            return Err(Message(format!(
                "You don't see {} in here. I'm beginning to question your sanity",
                other.unwrap_or_default()
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
    ) -> Result<String, EnnuiError> {
        let room = match rooms.get_mut(loc) {
            Some(r) => r,
            None => return Err(Simple(ItemNotFound)),
        };

        if let Some(Scenery(_)) = room.items().get_item(handle) {
            return Err(Simple(TooHeavy));
        }

        let player = match players.get_mut(&uuid) {
            Some(p) => p,
            None => {
                return Err(Fatal(
                    format!("Could not find primary player with uuid: {}", uuid),
                ))
            }
        };
        room.transfer(player, handle.into())
    }

    fn drop(
        rooms: &mut RoomList,
        players: &mut PlayerList,
        uuid: u128,
        loc: &Coord,
        handle: &str,
    ) -> Result<String, EnnuiError> {
        let room = match rooms.get_mut(loc) {
            Some(r) => r,
            None => {
                return Err(Fatal(
                    format!("unable to find room for player {} at {:?}", uuid, loc),
                ))
            }
        };
        let player = match players.get_mut(&uuid) {
            Some(p) => p,
            None => {
                return Err(Fatal(
                    format!("unable to find player {}", uuid),
                ))
            }
        };
        player.transfer(room, handle)
    }

    fn give(
        players: &mut PlayerList,
        rooms: &mut RoomList,
        loc: &Coord,
        ids: (u128, u128),
        other_name: Option<&str>,
        handle: &str,
    ) -> Result<String, EnnuiError> {
        let (uuid, other_id) = ids;
        let item = {
            let p = match players.get_mut(&uuid) {
                Some(p) => p,
                None => {
                    return Err(Fatal(
                        format!("unable to find player {}", uuid),
                    ))
                }
            };

            p.items_mut().get_item_owned(handle)?
        };

        let item_name = item.name().to_owned();
        let other_p = match players.get_mut(&other_id) {
            Some(p) => p,
            None => {
                let room = match rooms.get_mut(&loc) {
                    Some(r) => r,
                    None => {
                        return Err(Fatal(
                            format!(
                                "unable to find other player {:?} in room at {:?}",
                                other_id, loc
                            ),
                        ));
                    }
                };
                match room.get_item_mut(other_name.unwrap_or_default()) {
                    Some(Item::Guard(_, guard)) => {
                        use std::result::Result::*;
                        match guard.insert_item(item) {
                            Ok(()) => {
                                return Err(Message(format!("you see {} relax a little bit. maybe now they'll let you through",
                                                           article(guard.name()))

                                ));
                            }
                            Err(given_back) => {
                                let name = given_back.name().to_owned();
                                match players.get_mut(&uuid) {
                                    Some(p) => {
                                        if p.insert_item(given_back).is_err() {
                                            return Err(EnnuiError::Fatal(
                                                "wasn't able to return item to player after \
                                                    failed transfer to guard type."
                                                    .to_owned(),
                                            ));
                                        }
                                    }
                                    None => (),
                                }
                                return Err(Message(format!(
                                    "uh.. I don't think {} can accept your {}",
                                    article(guard.name()),
                                    name
                                )));
                            }
                        };
                    }
                    _ => {
                        return Err(Message(format!(
                            "you don't see {} here!",
                            other_name.unwrap_or_default()
                        )));
                    }
                }
            }
        };

        if other_p.items_mut().insert_item(item).is_err() {
            return Err(Fatal(
                format!(
                    "COULD NOT RETURN ITEM {} TO OTHER PLAYER {}",
                    item_name,
                    other_p.uuid()
                ),
            ));
        };
        Ok(item_name)
    }

    fn wear(players: &mut PlayerList, uuid: u128, handle: &str) -> Result<String, EnnuiError> {
        
        let (items, clothing) = { Self::get_player_mut(players, uuid)?.all_items_mut() };
        match items.get_item(handle) {
            Some(i) if i.is(Quality::Clothing) => (),
            None => return Err(Simple(ItemNotFound)),
            _ => return Err(Simple(NotClothing)),
        }
        items.transfer(clothing, handle)
    }

    fn remove(players: &mut PlayerList, uuid: u128, handle: &str) -> Result<String, EnnuiError> {
        let (items, clothing) = { Self::get_player_mut(players, uuid)?.all_items_mut() };
        clothing.transfer(items, handle)
    }

    fn get_player_mut(players: &mut PlayerList, uuid: u128) -> Result<&mut Player, EnnuiError> {
        match players.get_mut(&uuid) {
            Some(p) => Ok(p),
            None => Err(Fatal(
                format!("UNABLE TO FIND PLAYER {}", uuid),
            )),
        }
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
                            match room.get_item(other.unwrap_or_default()) {
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
