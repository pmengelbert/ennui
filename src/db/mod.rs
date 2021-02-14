use postgres::{Client, NoTls};
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

pub fn recipe_to_item(r: &crate::soul::recipe::Recipe) -> crate::item::Item {
    let mut db = DB::new().unwrap();

    let crate::soul::recipe::Recipe {
        combat_req,
        crafting_req,
        exploration_req,
    } = r;

    let (combat_req, crafting_req, exploration_req) = (
        *combat_req as i32,
        *combat_req as i32,
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
            i.itemid = (select itemid from ennui.recipe
            where
                crafting_req = $1,
                exploration_req = $2,
                combat_req = $3,
            )
        ",
            &[&crafting_req, &exploration_req, &combat_req],
        )
        .unwrap();

    if results.is_empty() {
        todo!()
    }

    let r1 = results.get(1).unwrap();
    let name: String = r1.get(0);
    let display: String = r1.get(1);
    let description: String = r1.get(2);
    let hook_strings: Vec<String> = r1.get(3);
    let attr: Vec<i32> = r1.get(4);

    todo!()
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
