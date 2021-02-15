use crate::error::{CmdErr, EnnuiError};
use std::convert::TryFrom;

pub enum Blank {}

/// `Recipe` is a combination of different souls. When combined, by the player,
/// using the `combine` command, the Recipe database is consulted for a matching
/// item template. This template generates an instance of the item, which is then
/// given to the player.
#[derive(Default, Debug)]
pub struct Recipe {
    pub combat_req: usize,
    pub crafting_req: usize,
    pub exploration_req: usize,
}

/// The text format of a recipe is always in the following format. Note that the
/// souls may be in any order. But the number of souls must always be specified.
/// Here is the format:
/// ```
///  > combine 0.crafting 1.exploration 2.combat
///
/// or
///
///  > combine 1.combat 2.crafting 0.exploration
/// ```
/// Note that even when the number is zero, it must be specified.
// TODO: does this belong in the `text` module?
impl TryFrom<&[&str]> for Recipe {
    type Error = EnnuiError;

    fn try_from(o: &[&str]) -> Result<Self, Self::Error> {
        if o.len() != 3 {
            return Err(EnnuiError::Simple(CmdErr::ItemNotFound));
        }

        let mut ret = Recipe::default();

        for s in o {
            let dot = match s.find('.') {
                None => return Err(EnnuiError::Simple(CmdErr::ItemNotFound)),
                Some(n) => n,
            };

            let number: usize = match s[..dot].parse() {
                Ok(n) => n,
                Err(_) => return Err(EnnuiError::Simple(CmdErr::ItemNotFound)),
            };

            match s.get(dot + 1..) {
                Some(s) if "crafting".starts_with(s) => ret.crafting_req += number,
                Some(s) if "combat".starts_with(s) => ret.combat_req += number,
                Some(s) if "exploration".starts_with(s) => ret.exploration_req += number,
                _ => return Err(EnnuiError::Simple(CmdErr::ItemNotFound)),
            }
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod recipe_test {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_try_from() {
        let x = "1.crafting 2.combat 3.exploration";

        let mut xv = vec![];
        let xz = x.split_whitespace();
        xv.extend(xz);
        let r: Recipe = (&xv[..]).try_into().unwrap();
        assert_eq!(r.crafting_req, 1);
        assert_eq!(r.combat_req, 2);
        assert_eq!(r.exploration_req, 3);

        // Should work out of order
        let mut xv = vec![];
        let x = "2.combat 1.crafting  3.exploration";
        let xz = x.split_whitespace();
        xv.extend(xz);
        let r: Recipe = (&xv[..]).try_into().unwrap();
        assert_eq!(r.crafting_req, 1);
        assert_eq!(r.combat_req, 2);
        assert_eq!(r.exploration_req, 3);

        let mut xv = vec![];
        let x = "0.combat 2.crafting 10.exploration";
        let xz = x.split_whitespace();
        xv.extend(xz);
        let r: Recipe = (&xv[..]).try_into().unwrap();
        assert_eq!(r.crafting_req, 2);
        assert_eq!(r.combat_req, 0);
        assert_eq!(r.exploration_req, 10);
    }
}
