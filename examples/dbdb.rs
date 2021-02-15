use bytes::{BufMut, BytesMut};
use postgres::{
    types::{to_sql_checked, FromSql, IsNull, ToSql, Type},
    Client, NoTls,
};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Copy)]
pub enum SoulKind {
    Combat = 1,
    Crafting = 2,
    Exploration = 3,
}

impl TryFrom<i32> for SoulKind {
    type Error = Box<dyn std::error::Error + Sync + Send>;

    fn try_from(o: i32) -> Result<Self, Self::Error> {
        match o {
            x if x == SoulKind::Combat as i32 => Ok(SoulKind::Combat),
            x if x == SoulKind::Crafting as i32 => Ok(SoulKind::Crafting),
            x if x == SoulKind::Exploration as i32 => Ok(SoulKind::Exploration),
            _ => Err(Box::new(std::fmt::Error)),
        }
    }
}

impl ToSql for SoulKind {
    to_sql_checked!();

    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let i = *self as u32;
        out.put_u32(i);

        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool
    where
        Self: Sized,
    {
        match ty {
            &Type::INT4 => true,
            _ => false,
        }
    }
}

impl<'a> FromSql<'a> for SoulKind {
    fn accepts(ty: &Type) -> bool {
        match ty {
            &Type::INT4 => true,
            _ => false,
        }
    }

    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let mut i = 0_i32;

        let (lst, first) = raw.split_last().unwrap_or((&0, &[]));
        for x in first {
            i |= *x as i32;
            i = i << 8;
        }
        i |= *lst as i32;

        i.try_into()
    }
}

fn main() {
    let mut client =
        Client::connect("host=localhost user=postgres dbname=exercises", NoTls).unwrap();

    let x = client
        .execute(
            "update ennui.item set kind = $1 where itemid = 1",
            &[&SoulKind::Crafting],
        )
        .unwrap();

    let y: SoulKind = client
        .query("select kind from ennui.item where itemid = 1", &[])
        .unwrap()[0]
        .get(0);

    println!("{:#?}", y);
}
