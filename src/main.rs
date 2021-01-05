use ennui::game::{Game, GameResult};
use ennui::player::{Player, Uuid};
use std::io::Write;
use ennui::text::Wrap;
use ennui::text::message::MessageFormat;
use ennui::text::Color::Magenta;

fn main() -> GameResult<()> {
    let mut g = Game::new()?;
    let p = Player::new();

    let uuid = p.uuid();
    g.add_player(p);

    loop {
        let mut s = String::new();
        std::io::stdin().read_line(&mut s)?;

        let s = s.trim();

        if let Ok((aud, msg)) = g.interpret(uuid, s) {
            println!(
                "{}",
                msg.to_self().wrap(90).color(Magenta)
            );
        } else {
            break;
        }
    }

    Ok(())
}
