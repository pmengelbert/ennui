use ennui::game::{Game, GameResult};
use ennui::player::{Player, Uuid};
use std::io::Write;

fn main() -> GameResult<()> {
    let mut g = Game::new()?;
    let p = Player::new();

    let uuid = p.uuid();
    g.add_player(p);

    loop {
        let mut s = String::new();
        std::io::stdout().write_all(b"\n > ")?;
        std::io::stdout().flush()?;

        std::io::stdin().read_line(&mut s)?;

        let s = s.trim();

        if let Ok((aud, msg)) = g.interpret(uuid, s) {
            println!(
                "\n\n{:?}\n{:?}\n{}\n{:?}\n\n",
                aud.id(),
                aud.others(),
                msg.to_self(),
                msg.to_others()
            );
        } else {
            break;
        }
    }

    Ok(())
}
