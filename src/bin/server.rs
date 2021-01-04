use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

use ennui::error::EnnuiError;

use ennui::game::{Game, GameResult};
use ennui::player::{Player, Uuid};
use ennui::text::message::{Audience, Broadcast, FightAudience, Msg};
use ennui::text::Color::{Green, Red};

use ennui::fight::FightMessage;
use ennui::text::channel::{MessageHandler, MessageReceiver};
use std::sync::mpsc::channel;

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
        let n = match self.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                println!("{}", e);
                0
            }
        };
        Ok(
            String::from_utf8((&buf[..n.checked_sub(1).unwrap_or_default()]).to_owned())
                .unwrap_or_default(),
        )
    }
}

fn main() -> GameResult<()> {
    let listener = TcpListener::bind("0.0.0.0:8089")?;

    let g = Game::new()?;
    let shared_game = arc_mutex!(g);

    let (sender, receiver) = channel::<JoinHandle<std::io::Result<()>>>();
    spawn(move || {
        for handle in receiver {
            match handle.join() {
                Ok(_) => (),
                Err(err) => {
                    println!("[{}]: {:#?}", Red("ERROR".to_owned()), err);
                }
            }
        }
    });

    let (fight_sender, fight_receiver) = channel::<(FightAudience, FightMessage)>();
    let rcv = MessageReceiver(fight_receiver);
    rcv.start(shared_game.clone());
    shared_game.lock().unwrap().set_sender(fight_sender);

    for stream in listener.incoming() {
        let game_clone = shared_game.clone();

        let stream = stream?;

        let p = Player::new_with_stream(stream);
        let uuid = p.uuid();
        {
            let mut game = match game_clone.lock() {
                Ok(g) => g,
                Err(err) => {
                    println!("error: {}", err);
                    continue;
                }
            };
            game.add_player(p);
        }

        sender
            .send(spawn(move || handle_client(uuid, game_clone)))
            .unwrap();
    }

    Ok(())
}

fn handle_client(p: u128, g: Arc<Mutex<Game>>) -> std::io::Result<()> {
    get_and_set_player_name(p, g.clone())?;

    println!("[{}]: player named", Green("SUCCESS".to_owned()));
    loop {
        let s = get_player_command(p, g.clone())?;

        println!("[{}]: Got player command", Green("SUCCESS".to_owned()));
        {
            let mut g = match g.lock() {
                Ok(g) => g,
                Err(err) => {
                    println!("error: {}", err);
                    break;
                }
            };

            let resp = g.interpret(p, &s);
            println!("[{}]: Got response", Green("SUCCESS".to_owned()));
            match resp {
                Ok((aud, msg)) => {
                    let results = g.send(&*aud, &*msg);
                    for (id, result) in results {
                        match result {
                            Err(e) => {
                                println!("[{}]: {:?}", Red("ERROR".to_owned()), e);
                                let p = g.remove_player(id).unwrap_or_default();

                                std::mem::drop(p);
                            }
                            _ => (),
                        }
                    }
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

fn get_player_command(p: u128, g: Arc<Mutex<Game>>) -> std::io::Result<String> {
    let mut stream = {
        let mut g = g.lock().map_err(|_| std::io::ErrorKind::AddrNotAvailable)?;
        let p = g
            .players_mut()
            .get_mut(&p)
            .ok_or(std::io::ErrorKind::NotFound)?
            .clone();

        let p = p.lock().unwrap();
        p.clone_stream().ok_or(std::io::ErrorKind::NotFound)?
    };

    stream.read_line()
}

fn get_and_set_player_name(p: u128, g: Arc<Mutex<Game>>) -> std::io::Result<()> {
    let clone = g.clone();
    let result = spawn(move || {
        let mut g = clone.lock().unwrap();

        std::io::Result::Ok({
            let p = g
                .players_mut()
                .get_mut(&p)
                .ok_or(std::io::ErrorKind::NotFound)?
                .clone();

            let p = p.lock().unwrap();
            p.clone_stream().ok_or(std::io::ErrorKind::NotFound)?
        })
    });

    let mut stream = result
        .join()
        .map_err(|_| std::io::ErrorKind::AddrNotAvailable)?
        .map_err(|_| std::io::ErrorKind::AddrNotAvailable)?;

    stream.write(b"enter your name: ")?;
    let name = stream.read_line()?;
    stream.write(b" > ")?;

    let mut g = g.lock().unwrap();
    let res = g
        .set_player_name(p, &name)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::NotFound));
    g.announce_player(p);
    res
}
