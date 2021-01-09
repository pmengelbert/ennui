use {
    crate::game, crate::player::Player, crate::text::Wrap, lazy_static::lazy_static,
    mut_static::MutStatic, std::sync::Arc, std::sync::Mutex, wasm_bindgen::prelude::*,
};

lazy_static! {
    pub static ref GAME: MutStatic<Arc<Mutex<game::Game>>> = {
        let mut g = game::Game::new().unwrap();
        let mut p = Player::new();
        p.set_name("peter");
        g.add_player(p);
        MutStatic::from(Arc::new(Mutex::new(g)))
    };
}

#[wasm_bindgen]
pub fn interpret(s: &str) -> String {
    let g = GAME.read().unwrap();
    let mut g = g.lock().unwrap();

    let x = g.interpret(10, s);
    let ret = match x {
        Ok(s) => s.1.to_self().wrap(80),
        Err(e) => {
            format!("{:?}", e)
        }
    };
    ret
}

pub fn new_player_id() -> u128 {
    10
}
