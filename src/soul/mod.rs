pub mod recipe;

use crate::attribute::Quality;
use crate::describe::Describe;
use crate::handle;
use crate::hook::Hook;
use crate::list::{List, ListTrait};
use std::convert::TryFrom;
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoulKind {
    None = 0,
    Combat = 1,
    Crafting = 2,
    Exploration = 3,
}

impl Default for SoulKind {
    fn default() -> Self {
        Self::None
    }
}

impl TryFrom<i32> for SoulKind {
    type Error = Box<dyn Error + Sync + Send>;

    fn try_from(o: i32) -> Result<Self, Self::Error> {
        match o {
            x if x == SoulKind::None as i32 => Ok(SoulKind::Combat),
            x if x == SoulKind::Combat as i32 => Ok(SoulKind::Combat),
            x if x == SoulKind::Crafting as i32 => Ok(SoulKind::Crafting),
            x if x == SoulKind::Exploration as i32 => Ok(SoulKind::Exploration),
            _ => Err(Box::new(std::fmt::Error)),
        }
    }
}

impl Describe for SoulKind {
    fn handle(&self) -> Hook {
        match self {
            SoulKind::Combat => handle![combat, soul, warrior],
            SoulKind::Crafting => handle![crafting, soul, crafter],
            SoulKind::Exploration => handle![exploration, soul, explorer],
            SoulKind::None => handle![],
        }
    }

    fn name(&self) -> String {
        match self {
            SoulKind::Combat => "combat soul",
            SoulKind::Crafting => "crafting soul",
            SoulKind::Exploration => "exploration soul",
            SoulKind::None => "",
        }
        .into()
    }

    fn display(&self) -> String {
        match self {
            SoulKind::Combat => "A warrior soul wisps its way through the air",
            SoulKind::Crafting => "The soul of a maker travels through the air currents",
            SoulKind::Exploration => {
                "An explorer's soul moves gently through every nook and cranny of the room"
            }
            SoulKind::None => "",
        }
        .into()
    }

    fn description(&self) -> String {
        todo!()
    }
}

impl List<SoulKind, Quality> {
    pub fn process_recipe(&mut self, recipe: &recipe::Recipe) -> bool {
        let list = self.list();

        let crafting = list
            .iter()
            .filter(|s| matches!(s, SoulKind::Crafting))
            .count();
        let exploration = list
            .iter()
            .filter(|s| matches!(s, SoulKind::Exploration))
            .count();
        let combat = list
            .iter()
            .filter(|s| matches!(s, SoulKind::Combat))
            .count();

        match recipe {
            recipe::Recipe {
                crafting_req,
                exploration_req,
                combat_req,
            } if crafting >= *crafting_req
                && exploration >= *exploration_req
                && combat >= *combat_req =>
            {
                for _ in 0..*crafting_req {
                    self.get_item_owned("crafting".into());
                }

                for _ in 0..*exploration_req {
                    self.get_item_owned("exploration".into());
                }

                for _ in 0..*combat_req {
                    self.get_item_owned("combat".into());
                }

                true
            }
            _ => false,
        }
    }
}
