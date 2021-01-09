#![feature(try_trait)]
#![feature(assoc_char_funcs)]

pub mod error;
pub mod fight;
pub mod game;
mod interpreter;
mod item;
pub mod map;
pub mod player;
pub mod text;

use lazy_static::lazy_static;
use mut_static::MutStatic;
use std::sync::Mutex;
use std::sync::Arc;

//type EnnuiResult = Result<String, EnnuiError>;
#[macro_export]
macro_rules! arc_mutex(
    ($wrapped:expr) => {
        Arc::new(Mutex::new($wrapped))
    };
);

lazy_static! {
    pub static ref GAME: MutStatic<Arc<Mutex<game::Game>>> = {
        let mut g = game::Game::new().unwrap();
        let mut p = Player::new();
        p.set_name("peter");
        g.add_player(p);
        MutStatic::from(Arc::new(Mutex::new(g)))
    };
}

use wasm_bindgen::prelude::*;
use crate::player::{Player, Uuid};
use crate::error::EnnuiError;
use crate::text::message::{Message, Messenger};
use crate::text::Wrap;

#[wasm_bindgen]
pub fn interpret(s: &str) -> String {
    let g = GAME.read().unwrap();
    let mut g = g.lock().unwrap();

    let x = g.interpret(10, s);
    let ret = match x {
        Ok(s) => {
            s.1.to_self().wrap(80)
        }
        Err(e) => {
            format!("{:?}", e)
        }
    };
    ret
}

type WriteResult = std::io::Result<usize>;

pub struct SendError {
    pub result: std::io::Result<usize>,
    pub pid: u128,
}

