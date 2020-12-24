#![feature(try_trait)]
#![feature(assoc_char_funcs)]

#[macro_export]
macro_rules! arc_mutex(
    ($wrapped:expr) => {
        Arc::new(Mutex::new($wrapped))
    };
);

type PassFail = Result<(), std::option::NoneError>;

pub mod game;
pub mod interpreter;
pub mod item;
pub mod map;
pub mod player;
pub mod text;
