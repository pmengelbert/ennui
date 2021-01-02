#![feature(try_trait)]
#![feature(assoc_char_funcs)]

#[macro_export]
macro_rules! arc_mutex(
    ($wrapped:expr) => {
        Arc::new(Mutex::new($wrapped))
    };
);

type WriteResult = std::io::Result<usize>;

pub mod game;
mod interpreter;
mod item;
pub mod map;
pub mod player;
mod text;
pub mod error;
