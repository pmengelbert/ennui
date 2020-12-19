use ennui::game::Game;
use ennui::player::Player;
use std::io::Write;

fn main() {
    let mut g = Game::new();
    let mut p = Player::new("peter");
    let mut s = String::new();
    std::io::stdout()
        .write_all(b"> ");
    std::io::stdout().flush();

    std::io::stdin()
        .read_line(&mut s)
        .expect("failure");

    let s = s.trim();

    let uuid = p.uuid();
    g.add_player(p);
    let ret = g.interpret(uuid, s).unwrap();
    println!("{}", ret);
}
