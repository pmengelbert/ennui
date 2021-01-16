use super::Player;
use crate::error::EnnuiError;
use crate::interpreter::{CommandFunc, CommandMessage};
use crate::item::Description;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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
}

pub struct Npc {
    player: Player,
    ai_type: AI,
}

impl From<YamlPlayer> for Npc {
    fn from(other: YamlPlayer) -> Self {
        let YamlPlayer { info, ai_type } = other;

        let ai_type = ai_type.unwrap_or(AI::Static);
        let mut p = Player::new();
        p.info = info;

        Self { player: p, ai_type }
    }
}

impl Npc {
    pub fn new(player: Player, ai_type: AI) -> Self {
        Self { player, ai_type }
    }
    pub fn init(&mut self) {}
}

fn talker(g: &mut crate::game::Game, v: &[String]) -> CommandFunc {
    g.interpreter()
        .commands()
        .lock()
        .unwrap()
        .get(&crate::interpreter::CommandKind::Say)
        .unwrap()
        .clone()
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
