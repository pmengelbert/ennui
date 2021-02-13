use std::convert::TryFrom;
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub enum SoulKind {
    Combat = 1,
    Crafting = 2,
    Exploration = 3,
}

impl TryFrom<i32> for SoulKind {
    type Error = Box<dyn Error + Sync + Send>;

    fn try_from(o: i32) -> Result<Self, Self::Error> {
        match o {
            x if x == SoulKind::Combat as i32 => Ok(SoulKind::Combat),
            x if x == SoulKind::Crafting as i32 => Ok(SoulKind::Crafting),
            x if x == SoulKind::Exploration as i32 => Ok(SoulKind::Exploration),
            _ => Err(Box::new(std::fmt::Error)),
        }
    }
}
