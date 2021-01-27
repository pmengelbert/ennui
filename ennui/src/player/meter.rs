use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MeterKind {
    Hit(Meter),
    Mana(Meter),
    Movement(Meter),
    Strength(Meter),
    Dexterity(Meter),
    Weight(Meter),
    Height(Meter),
}

impl Display for MeterKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.string().to_uppercase(),
            self.safe_unwrap()
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Meter(pub i64, pub i64);

impl Display for Meter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} / {}]", self.0, self.1)
    }
}

impl Meter {
    pub fn current(&self) -> i64 {
        self.0
    }

    pub fn max(&self) -> i64 {
        self.1
    }

    pub fn set(&mut self, val: i64) {
        self.0 = val;
    }
}

impl MeterKind {
    pub fn current(&self) -> i64 {
        self.safe_unwrap().0
    }

    pub fn max(&self) -> i64 {
        self.safe_unwrap().1
    }

    pub fn set(&mut self, val: i64) {
        *self.safe_unwrap_mut().0 = val
    }

    fn safe_unwrap(&self) -> &Meter {
        use MeterKind::*;
        match self {
            Hit(m) | Mana(m) | Movement(m) | Strength(m) | Dexterity(m) | Weight(m) | Height(m) => {
                m
            }
        }
    }

    fn string(&self) -> &'static str {
        match self {
            MeterKind::Hit(_) => "hit",
            MeterKind::Mana(_) => "mana",
            MeterKind::Movement(_) => "movement",
            MeterKind::Strength(_) => "strength",
            MeterKind::Dexterity(_) => "dexterity",
            MeterKind::Weight(_) => "weight",
            MeterKind::Height(_) => "height",
        }
    }

    fn safe_unwrap_mut(&mut self) -> (&mut i64, &mut i64) {
        use MeterKind::*;
        match self {
            Hit(m) | Mana(m) | Movement(m) | Strength(m) | Dexterity(m) | Weight(m) | Height(m) => {
                (&mut m.0, &mut m.1)
            }
        }
    }
}
