use bytes::{BufMut, BytesMut};
use postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use std::convert::TryInto;

use crate::soul::SoulKind;

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
