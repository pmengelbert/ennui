#![feature(try_trait)]
#![feature(assoc_char_funcs)]

#[macro_export]
macro_rules! arc_mutex(
    ($wrapped:expr) => {
        Arc::new(Mutex::new($wrapped))
    };
);

type WriteResult = std::io::Result<usize>;

pub struct SendError {
    pub result: std::io::Result<usize>,
    pub pid: u128,
}

pub mod error;
pub mod fight;
pub mod game;
mod interpreter;
mod item;
pub mod map;
pub mod player;
pub mod text;
