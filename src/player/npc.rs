use super::{Player, PlayerType, Quality};
use crate::describe::Description;
use crate::item::{list::ItemList, list::ItemListTrout, Item};
use crate::map::coord::Coord;
use crate::text::message::Broadcast;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AI {
    Static,
    Talker(Vec<String>),
    Walker,
}

#[derive(Deserialize, Serialize)]
pub struct YamlPlayer {
    #[serde(flatten)]
    pub info: Description,
    #[serde(default)]
    pub ai_type: Option<AI>,
    #[serde(default)]
    pub loc: Coord,
}

#[derive(Debug, Default)]
pub struct Npc {
    player: Player,
    ai_type: Option<AI>,
    tx: Option<Mutex<Sender<NpcMessage>>>,
}

impl From<PlayerType> for Item {
    fn from(mut other: PlayerType) -> Self {
        let player = match &mut other {
            PlayerType::Npc(npc) => {
                npc.stop();
                let npc = std::mem::take(npc);
                let Npc { player, .. } = npc;
                player
            }
            PlayerType::Human(p) | PlayerType::Dummy(p) => std::mem::take(p),
        };

        player.into()
    }
}

impl From<Player> for Item {
    fn from(other: Player) -> Self {
        let Player {
            info,
            mut attr,
            mut items,
            mut clothing,
            ..
        } = other;

        let mut attributes = attr;

        let Description {
            name, mut handle, ..
        } = info;

        handle.push("corpse".into());
        let display = format!("The corpse of {} lies here, decomposing", name);
        let description = format!("Where once stood {}, now lies a rotting corpse", name);
        attributes.push(Quality::Scenery);

        let d = Description {
            name,
            display,
            handle,
            description,
        };

        let i2 = crate::item::Item2 {
            info: d.clone(),
            attr: attributes,
        };

        let mut new_items = ItemList::new_with_info(d);
        for item in items.iter_mut() {
            let item = std::mem::take(item);
            new_items.push(item);
        }

        for item in clothing.iter_mut() {
            let item = std::mem::take(item);
            new_items.push(item);
        }

        Item::Container(i2, Box::new(new_items))
    }
}

impl From<YamlPlayer> for Npc {
    fn from(other: YamlPlayer) -> Self {
        let YamlPlayer { info, ai_type, loc } = other;

        let ai_type = ai_type;
        let mut p = Player::new();
        p.loc = loc;
        p.info = info;

        Self {
            player: p,
            ai_type,
            tx: None,
        }
    }
}

pub enum NpcMessage {
    Stop,
}

impl Npc {
    pub fn new(player: Player, ai_type: AI) -> Self {
        let ai_type = Some(ai_type);
        Self {
            player,
            ai_type,
            tx: None,
        }
    }

    pub fn init(&mut self, g: Arc<Mutex<crate::game::Game>>) {
        let id = self.player.uuid;
        let ai_type = self.ai_type.take().unwrap_or(AI::Static);
        let (tx, rx) = channel::<NpcMessage>();
        self.tx = Some(Mutex::new(tx));

        match ai_type {
            AI::Static => (),
            AI::Talker(v) => {
                let v = v.clone();
                thread::spawn(move || loop {
                    let interval: u64 = rand::thread_rng().gen_range(20, 30);
                    std::thread::sleep(std::time::Duration::new(interval, 0));
                    match rx.try_recv() {
                        Ok(NpcMessage::Stop) | Err(TryRecvError::Disconnected) => break,
                        _ => (),
                    }
                    let n: usize = rand::thread_rng().gen_range(0, v.len());
                    let phrase = &v[n];
                    eprintln!("PHRASE: {}", phrase);
                    eprintln!("in file {} on line number {}", file!(), line!());

                    let mut command = String::with_capacity(4 + phrase.len());
                    command.push_str("say ");
                    command.push_str(phrase);
                    eprintln!("command: {}", command);
                    eprintln!("in file {} on line number {}", file!(), line!());

                    let (aud, msg) = g
                        .lock()
                        .unwrap()
                        .interpret(id, &command)
                        .expect("HANDLE THIS BETTER");
                    g.lock().unwrap().send(&*aud, &*msg);
                });
                ()
            }
            AI::Walker => {
                thread::spawn(move || loop {
                    let interval: u64 = rand::thread_rng().gen_range(20, 30);
                    std::thread::sleep(std::time::Duration::new(interval, 0));
                    match rx.try_recv() {
                        Ok(NpcMessage::Stop) | Err(TryRecvError::Disconnected) => break,
                        _ => (),
                    }
                    let n: usize = rand::thread_rng().gen_range(0, 4);
                    let command = match n {
                        0 => "n",
                        1 => "s",
                        2 => "e",
                        _ => "w",
                    };
                    let (aud, msg) = g
                        .lock()
                        .unwrap()
                        .interpret(id, command)
                        .expect("HANDLE THIS BETTER");
                    g.lock().unwrap().send(&*aud, &*msg);
                });
                ()
            }
        }
    }

    pub fn stop(&mut self) -> Option<()> {
        self.tx.as_ref()?.lock().ok()?.send(NpcMessage::Stop).ok()?;
        Some(())
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }
}

#[cfg(test)]
mod npc_test {
    use super::*;
    use crate::player::PlayerType;

    #[test]
    fn yaml_import_test() {
        let x = r#"---
ai_type: 
  Talker:
  - shut up
  - get out
name: Bill
handle: ["bill", "guy"]
description: ""
display:
  Bill is here, just minding his own business
"#;
        let p: YamlPlayer = serde_yaml::from_str(x).unwrap();
        assert_eq!(p.info.name, "Bill");
        assert_eq!(p.info.handle, "bill");
        assert_eq!(p.info.handle, "guy");
        assert!(matches!(p.ai_type, Some(AI::Talker(_))));
        let r: PlayerType = p.into();
        assert!(matches!(r, PlayerType::Npc(_)));

        let y = r#"---
name: Bill
handle: ["bill", "guy"]
description: ""
display:
  Bill is here, just minding his own business
"#;

        let q: YamlPlayer = serde_yaml::from_str(y).unwrap();
        assert_eq!(q.ai_type, None);
        let s: PlayerType = q.into();
        assert!(matches!(s, PlayerType::Human(_)));
    }
}
