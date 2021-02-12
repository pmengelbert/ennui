use postgres::{Client, NoTls};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Copy)]
pub enum SoulKind {
    Combat = 1,
    Crafting = 2,
    Exploration = 3,
}

impl SoulKind {
    fn i32(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for SoulKind {
    type Error = ();

    fn try_from(o: i32) -> Result<Self, Self::Error> {
        match o {
            1 => Ok(SoulKind::Combat),
            2 => Ok(SoulKind::Crafting),
            _ => Err(()),
        }
    }
}

fn main() {
    let mut client =
        Client::connect("host=localhost user=postgres dbname=exercises", NoTls).unwrap();

    let x = client
        .execute(
            "update ennui.item set kind = $1 where itemid = 1",
            &[&(SoulKind::Crafting.i32())],
        )
        .unwrap();

    let y: i32 = client
        .query("select kind from ennui.item where itemid = 1", &[])
        .unwrap()[0]
        .get(0);
    let y: SoulKind = y.try_into().unwrap();

    println!("{:#?}", y);
}
