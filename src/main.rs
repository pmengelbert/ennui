use ennui::game::{Game, GameResult};
use ennui::player::{Player, Uuid};
use std::io::Write;
use ennui::text::Wrap;
use ennui::text::message::MessageFormat;
use ennui::text::Color::Magenta;
use wasm_bindgen::prelude::*;
//use stdweb::js_export;

use ennui::error::EnnuiError;
//use wasm_bindgen::convert::IntoWasmAbi;
//use wasm_bindgen::describe::WasmDescribe;
use mut_static::MutStatic;
use std::sync::Mutex;
use std::sync::Arc;
use ennui::arc_mutex;
use lazy_static::lazy_static;

type EnnuiResult = Result<String, EnnuiError>;

lazy_static! {
    pub static ref GAME: MutStatic<Arc<Mutex<Game>>> = {
        MutStatic::from(arc_mutex!(Game::new().unwrap()))
    };
}

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
    if g.players_mut().len() == 0 {
        let mut p = Player::new();
        id = p.uuid();
        p.set_name("peter");
        g.add_player(p);
    }
    let x = g.interpret(id, s).unwrap();
    x.1.to_self()
}

// fn main() -> GameResult<()> {
//     let mut g = Game::new()?;
//     let p = Player::new();
//
//     let uuid = p.uuid();
//     g.add_player(p);
//
//     loop {
//         let mut s = String::new();
//         std::io::stdin().read_line(&mut s)?;
//
//         let s = s.trim();
//
//         if let Ok((aud, msg)) = g.interpret(uuid, s) {
//             println!(
//                 "{}",
//                 msg.to_self().wrap(90).color(Magenta)
//             );
//         } else {
//             break;
//         }
//     }
//
//     Ok(())
// }
