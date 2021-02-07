#![feature(try_trait)]
#![feature(assoc_char_funcs)]
#![feature(backtrace)]

mod db;
pub mod describe;
pub mod error;
pub mod fight;
pub mod game;
pub mod gram_object;
mod interpreter;
mod item;
pub mod map;
pub mod player;
pub mod text;

//type EnnuiResult = Result<String, EnnuiError>;
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
