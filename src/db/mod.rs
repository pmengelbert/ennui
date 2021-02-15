use crate::attribute::Quality;
use crate::describe::Description;
use crate::hook::Hook;
use crate::item::DescriptionWithQualities;
use postgres::{Client, NoTls};
use std::convert::TryInto;
mod sql;

#[derive(Debug, Clone)]
pub enum DBError {
    NoRows,
}

impl std::fmt::Display for DBError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for DBError {}

pub struct DB {
    conn: Client,
}

impl DB {
    pub fn new() -> Result<Self, postgres::Error> {
        Ok(Self {
            conn: Client::connect("host=postgres user=postgres password=password123", NoTls)?,
        })
    }

    pub fn helpfile(&mut self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let it = self.conn.query(
            "SELECT title, description \
                FROM ennui.help \
                WHERE $1 = ANY(hook)",
            &[&name],
        )?;

        let title: &str = match it.get(0) {
            Some(row) => row,
            None => Err(DBError::NoRows)?,
        }
        .get(0);

        let desc: &str = match it.get(0) {
            Some(row) => row,
            None => Err(DBError::NoRows)?,
        }
        .get(1);

        let mut ret = String::from(title);
        ret.push_str("\n\n");
        ret.push_str(desc);

        Ok(ret)
    }
}

pub fn recipe_to_item(r: &crate::soul::recipe::Recipe) -> Option<crate::item::Item> {
    let mut db = match DB::new() {
        Ok(db) => db,
        Err(_) => return None,
    };

    let crate::soul::recipe::Recipe {
        combat_req,
        crafting_req,
        exploration_req,
    } = r;

    let (combat_req, crafting_req, exploration_req) = (
        *combat_req as i32,
        *crafting_req as i32,
        *exploration_req as i32,
    );

    let results = db
        .conn
        .query(
            "\
        SELECT name, display, description, hook, attributes
        FROM
            ennui.item i
        WHERE
            i.itemid = (SELECT itemid FROM ennui.recipe
            WHERE
                crafting_req = $1 AND
                exploration_req = $2 AND
                combat_req = $3
            )
        ",
            &[&crafting_req, &exploration_req, &combat_req],
        )
        .unwrap();

    if results.is_empty() {
        return None;
    }

    let r1 = results.get(0).unwrap();
    let name: String = r1.get(0);
    let display: String = r1.get(1);
    let description: String = r1.get(2);
    let handle: Vec<String> = r1.get(3);
    let handle = Hook(handle);
    let attr: Vec<i32> = r1.get(4);

    let mut x: Vec<Quality> = vec![];

    for a in attr {
        match a.try_into() {
            Ok(z) => x.push(z),
            Err(_) => return None,
        }
    }

    let d = DescriptionWithQualities {
        info: Description {
            name,
            display,
            description,
            handle,
        },
        attr: x,
    };

    let z = crate::item::Item::Holdable(Box::new(d));

    Some(z)
}

#[cfg(test)]
mod db_test {
    use super::*;

    #[test]
    fn db_connect() {
        let mut db = DB::new().unwrap();

        let result = db.helpfile("look");
        assert!(dbg!(&result).is_ok());
        let result = result.unwrap();
        assert!(result.starts_with("LOOK"));

        let result2 = db.helpfile("examine");
        assert!(dbg!(&result2).is_ok());
        let result2 = result2.unwrap();
        assert_eq!(result, result2);

        let bad = db.helpfile("butts");
        assert!(bad.is_err());

        println!("{}", result2);
    }
}
