use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
pub trait Attribute<T: Copy + Eq> {
    fn attr(&self) -> Vec<T>;
    fn set_attr(&mut self, q: T);
    fn unset_attr(&mut self, q: T);

    fn is(&self, a: T) -> bool {
        self.attr().contains(&a)
    }

    fn is_all(&self, ats: &[T]) -> bool {
        for a in ats {
            if !self.attr().contains(a) {
                return false;
            }
        }

        true
    }
}

/// `Quality` is used to describe extra attributes on players, items and rooms
#[derive(Copy, Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub enum Quality {
    Default = 0,
    Clothing = 1,
    Weapon = 2,
    Scenery = 3,
    Edible = 4,
    Holdable = 5,
    Container = 6,
    Guard = 7,
    Key = 8,
}

impl TryFrom<i32> for Quality {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            q if q == Quality::Default as i32 => Ok(Quality::Default),
            q if q == Quality::Clothing as i32 => Ok(Quality::Clothing),
            q if q == Quality::Weapon as i32 => Ok(Quality::Weapon),
            q if q == Quality::Scenery as i32 => Ok(Quality::Scenery),
            q if q == Quality::Edible as i32 => Ok(Quality::Edible),
            q if q == Quality::Holdable as i32 => Ok(Quality::Holdable),
            q if q == Quality::Container as i32 => Ok(Quality::Container),
            q if q == Quality::Guard as i32 => Ok(Quality::Guard),
            q if q == Quality::Key as i32 => Ok(Quality::Key),
            _ => Err("could not make conversion".into()),
        }
    }
}

impl Default for Quality {
    fn default() -> Self {
        Self::Default
    }
}
