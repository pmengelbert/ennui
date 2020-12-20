use ennui::game::Game;
use ennui::player::Player;
use std::io::Write;

fn main() {
    let mut g = Game::new();
    let mut p = Player::new("peter");

    let uuid = p.uuid();
    g.add_player(p);

    loop {
        let mut s = String::new();
        std::io::stdout()
            .write_all(b"\n > ");
        std::io::stdout().flush();

        std::io::stdin()
            .read_line(&mut s)
            .expect("failure");
        let s = s.trim();

        if let Some(msg) = g.interpret(uuid, s) {
            println!("{}", msg);
        } else {
            println!("i don't know about that...")
        }
    }
}
