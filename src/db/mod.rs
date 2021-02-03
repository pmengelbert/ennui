use postgres::{Client, NoTls};

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
            conn: Client::connect("host=localhost user=pme dbname=exercises", NoTls)?,
        })
    }

    pub fn helpfile(&mut self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let it = self.conn.query(
            "SELECT title FROM ennui.help WHERE $1 = ANY(hook)",
            &[&name],
        )?;
        let data: &str = match it.get(0) {
            Some(row) => row,
            None => Err(DBError::NoRows)?,
        }
        .get(0);
        Ok(data.to_owned())
    }
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
        assert_eq!(result, "the look command");

        let result2 = db.helpfile("loo");
        assert!(dbg!(&result2).is_ok());
        let result2 = result2.unwrap();
        assert_eq!(result, result2);

        let bad = db.helpfile("butts");
        assert!(bad.is_err());
    }
}
