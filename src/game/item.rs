use crate::game::Game;

use crate::item::{Attribute, Describe, Item, Quality};
use crate::map::coord::Coord;
use crate::map::list::RoomList;
use crate::player::list::PlayerList;
use crate::player::{Player, Uuid};
use crate::text::article;

use crate::error::CmdErr::{ItemNotFound, NotClothing, PlayerNotFound, TooHeavy};
use crate::error::EnnuiError;
use crate::error::EnnuiError::{Fatal, Msg, Simple};
use crate::item::list::{Holder, ItemList, ListTrait};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
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
            return Err(Msg(format!(
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

        match room.get_item(handle) {
            Some(s) if s.is(Quality::Scenery) => return Err(Simple(TooHeavy)),
            _ => (),
        }

        let player = players.get_mut(&uuid).ok_or(Fatal(format!(
            "Could not find primary player with uuid: {}",
            uuid
        )))?;
        room.transfer(player.lock().unwrap().deref_mut(), handle.into())
    }

    fn drop(
        rooms: &mut RoomList,
        players: &mut PlayerList,
        uuid: u128,
        loc: &Coord,
        handle: &str,
    ) -> Result<String, EnnuiError> {
        let room = rooms.get_mut(loc).ok_or(Fatal(format!(
            "unable to find room for player {} at {:?}",
            uuid, loc
        )))?;
        let player = players
            .get_mut(&uuid)
            .ok_or(Fatal(format!("unable to find player {}", uuid)))?;
        player.lock().unwrap().transfer(room, handle)
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
            let p = players
                .get_mut(&uuid)
                .ok_or(Fatal(format!("unable to find player {}", uuid)))?;

            p.lock().unwrap().items_mut().get_item_owned(handle)?
        };

        let item_name = item.name().to_owned();

        let other_p = match players.get_mut(&other_id) {
            Some(p) => p,
            None => {
                let room = rooms.get_mut(&loc).ok_or(Fatal(format!(
                    "unable to find other player {:?} in room at {:?}",
                    other_id, loc
                )))?;
                return match room.get_item_mut(other_name.unwrap_or_default()) {
                    Some(Item::Guard(_, guard)) => match guard.insert_item(item) {
                        Ok(()) => Err(Msg(format!(
                            "you see {} relax a little bit. maybe now they'll let you through",
                            article(guard.name())
                        ))),
                        Err(given_back) => {
                            players
                                .get_mut(&uuid)
                                .ok_or(Fatal(
                                    "wasn't able to find the original player ...".to_owned(),
                                ))?
                                .lock()
                                .unwrap()
                                .insert_item(given_back)
                                .map_err(|_| {
                                    Fatal(
                                        "wasn't able to return item to player after failed transfer \
                                        to guard type."
                                            .to_owned(),
                                    )
                                })?;

                            Err(Msg(format!(
                                "I don't think {} can accept {}",
                                article(guard.name()),
                                article(&item_name)
                            )))
                        }
                    },
                    _ => Err(Msg(format!(
                        "you don't see {} here!",
                        other_name.unwrap_or_default()
                    ))),
                };
            }
        };

        if other_p
            .lock()
            .unwrap()
            .items_mut()
            .insert_item(item)
            .is_err()
        {
            return Err(Fatal(format!(
                "COULD NOT RETURN ITEM {} TO OTHER PLAYER {}",
                item_name,
                other_p.lock().unwrap().uuid()
            )));
        };
        Ok(item_name)
    }

    fn wear(players: &mut PlayerList, uuid: u128, handle: &str) -> Result<String, EnnuiError> {
        let p = Self::get_player_mut(players, uuid)?;
        let mut p = p.lock().unwrap();
        let (items, clothing) = p.all_items_mut();
        Self::check_if_clothing(handle, items)?;
        items.transfer(clothing, handle)
    }

    fn check_if_clothing(handle: &str, items: &mut ItemList) -> Result<(), EnnuiError> {
        match items.get_item(handle) {
            Some(i) if i.is(Quality::Clothing) => Ok(()),
            None => Err(Simple(ItemNotFound)),
            _ => Err(Simple(NotClothing)),
        }
    }

    fn remove(players: &mut PlayerList, uuid: u128, handle: &str) -> Result<String, EnnuiError> {
        let p = Self::get_player_mut(players, uuid)?;
        let mut p = p.lock().unwrap();
        let (items, clothing) = p.all_items_mut();
        clothing.transfer(items, handle)
    }

    fn get_player_mut(
        players: &mut PlayerList,
        uuid: u128,
    ) -> Result<Arc<Mutex<Player>>, EnnuiError> {
        match players.get_mut(&uuid) {
            Some(p) => Ok(p.clone()),
            None => Err(Fatal(format!("UNABLE TO FIND PLAYER {}", uuid))),
        }
    }

    fn validate_other_player(
        &self,
        other: Option<&str>,
        loc: &Coord,
        dir: Direction,
    ) -> Result<(), EnnuiError> {
        use Direction::*;

        let oid = self.id_of(other.unwrap_or_default());
        let other_id = oid.unwrap_or_default();

        let rooms = &self.rooms;

        if let Give = dir {
            match (other, oid) {
                (Some(o), None) => {
                    match rooms.get(loc) {
                        Some(room) => match room.get_item(o) {
                            Some(_) => {
                                return Ok(());
                            }
                            None => (),
                        },
                        None => (),
                    }

                    return Err(Simple(PlayerNotFound));
                }
                _ => (),
            }

            match self.loc_of(other_id) {
                Some(other_loc) if &other_loc != loc => {
                    return Err(Msg(format!(
                        "you don't see {} here !",
                        other.unwrap_or_default()
                    )));
                }
                None => {
                    return Ok(());
                }
                _ => (),
            }
        }

        Ok(())
    }
}
