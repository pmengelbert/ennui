use ennui::game::{Game, GameResult};
use ennui::player::{Player, Uuid};
use std::io::Write;

fn main() -> GameResult<()> {
    let mut g = Game::new()?;
    let p = Player::new("peter");

    let uuid = p.uuid();
    g.add_player(p);

    loop {
        let mut s = String::new();
        std::io::stdout().write_all(b"\n > ")?;
        std::io::stdout().flush()?;

        std::io::stdin().read_line(&mut s)?;

        let s = s.trim();

        if let Some(msg) = g.interpret(uuid, s) {
            println!("{}", msg);
        } else {
            break;
        }
    }

    Ok(())
}
