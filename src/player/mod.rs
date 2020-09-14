use self::Status::{Alive, Dead};

pub enum Status {
    Alive,
    Dead,
}

pub struct Player {
    name: String,
    status: Vec<Status>,
}
