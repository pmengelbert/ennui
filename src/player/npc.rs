use super::Player;
use crate::map::coord::Coord;
use crate::text::message::Broadcast;
use crate::item::Description;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use rand::Rng;

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AI {
    Static,
    Talker(Vec<String>),
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

#[derive(Debug)]
pub struct Npc {
    player: Player,
    ai_type: Option<AI>,
}

impl From<YamlPlayer> for Npc {
    fn from(other: YamlPlayer) -> Self {
        let YamlPlayer { info, ai_type, loc } = other;

        let ai_type = ai_type;
        let mut p = Player::new();
        p.loc = loc;
        p.info = info;

        Self { player: p, ai_type }
    }
}

impl Npc {
    pub fn new(player: Player, ai_type: AI) -> Self {
        let ai_type = Some(ai_type);
        Self { player, ai_type }
    }

    pub fn init(&mut self, g: Arc<Mutex<crate::game::Game>>) {
        let id = self.player.uuid;
        let ai_type = self.ai_type.take().unwrap_or(AI::Static);

        match ai_type {
            AI::Static => (),
            AI::Talker(v) => {
                let v = v.clone();
                std::thread::spawn(move || {
                    loop {
                        let interval: u64 = rand::thread_rng().gen_range(20, 30);
                        std::thread::sleep(std::time::Duration::new(interval, 0));
                        let n: usize = rand::thread_rng().gen_range(0, v.len());
                        let phrase = &v[n];
                        eprintln!("PHRASE: {}", phrase);
                        let mut command = String::with_capacity(4 + phrase.len());
                        command.push_str("say ");
                        command.push_str(phrase);
                        eprintln!("command: {}", command);
                        let (aud, msg) = g
                            .lock()
                            .unwrap()
                            .interpret(id, &command).expect("HANDLE THIS BETTER");
                        g.lock()
                            .unwrap()
                            .send(&*aud, &*msg);
                    }
                });
                ()
            }
        }
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
