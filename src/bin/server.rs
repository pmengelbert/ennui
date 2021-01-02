use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use ennui::error::EnnuiError;

use ennui::game::{Game, GameResult};
use ennui::player::{Player, Uuid};
use ennui::text::message::Broadcast;
use ennui::text::Color::Red;

macro_rules! arc_mutex(
    ($wrapped:expr) => {
        Arc::new(Mutex::new($wrapped))
    };
);

trait ReadLine {
    fn read_line(&mut self) -> std::io::Result<String>;
}

impl<T> ReadLine for T
where
    T: Read,
{
    fn read_line(&mut self) -> std::io::Result<String> {
        let mut buf = [0u8; 256];
        let n = self.read(&mut buf)?;
        Ok(String::from_utf8((&buf[..n - 1]).to_owned()).unwrap())
    }
}

fn main() -> GameResult<()> {
    let listener = TcpListener::bind("0.0.0.0:8089")?;

    let g = Game::new()?;
    let shared_game = arc_mutex!(g);
    let mut join_handles = vec![];

    for stream in listener.incoming() {
        let game_clone = shared_game.clone();

        let mut stream = stream?;
        write!(stream, "enter your name: ")?;
        let name = stream.read_line()?;

        let p = Player::new_with_stream(&name, stream.try_clone().unwrap());
        let uuid = p.uuid();
        {
            let mut game = match game_clone.lock() {
                Ok(g) => g,
                Err(err) => {
                    println!("error: {}", err);
                    continue;
                }
            };
            game.broadcast(format!("{} has joined the game.", name))?;
            game.add_player(p);
        }

        let stream_clone = stream.try_clone()?;

        join_handles.push(spawn(move || handle_client(stream_clone, uuid, game_clone)));
    }

    for handle in join_handles {
        handle.join().unwrap().unwrap_or_default();
    }

    Ok(())
}

fn handle_client<T: ReadLine + Write>(
    mut stream: T,
    p: u128,
    g: Arc<Mutex<Game>>,
) -> std::io::Result<()> {
    stream.write(b" > ")?;
    loop {
        let s = stream.read_line()?;

        {
            let mut g = match g.lock() {
                Ok(g) => g,
                Err(err) => {
                    println!("error: {}", err);
                    break;
                }
            };

            let resp = g.interpret(p, &s);
            match resp {
                Ok((aud, msg)) => {
                    g.send(&*aud, &*msg);
                }
                Err(e) => match e {
                    EnnuiError::Quit => {
                        g.remove_player(p);
                        break;
                    }
                    e => println!("[{}]: {:?}", Red("FATAL".to_owned()), e),
                },
            }
        }
    }

    Ok(())
}
