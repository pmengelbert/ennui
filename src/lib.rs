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
        MutStatic::from(Arc::new(Mutex::new(game::Game::new().unwrap())))
    };
}

use wasm_bindgen::prelude::*;
use crate::player::{Player, Uuid};
use crate::error::EnnuiError;
use crate::text::message::{Message, Messenger};


#[wasm_bindgen]
pub fn poo() -> String {
    "lol".into()
}

#[wasm_bindgen]
pub fn interpret(s: &str) -> String {
    let g = GAME.read().unwrap();
    let mut g = g.lock().unwrap();
    let mut id = 0_u128;
    let len = g.players_mut().len();
    if len == 0 {
        let mut p = Player::new();
        //return "here".into();
        id = p.uuid();
        p.set_name("peter");
        g.add_player(p);
    }

    let x = g.interpret(id, s);
    let ret = match x {
        Ok(s) => {
            s.1.to_self()
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

