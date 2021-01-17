use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

use ennui::error::EnnuiError;

use ennui::game::{Game, GameResult, NpcInit};
use ennui::player::{PlayerType, Player, Uuid};
use ennui::player::npc::{YamlPlayer};
use ennui::text::message::{Broadcast, FightAudience, MessageFormat};

use ennui::fight::FightMessage;
use ennui::text::channel::{MessageHandler, MessageReceiver};
use ennui::text::Color::{Green, Magenta, Red};
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
                eprintln!("{}", e);
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
                    eprintln!("[{}]: {:#?}", "ERROR".color(Red), err);
                }
            }
        }
    });

    let (fight_sender, fight_receiver) = channel::<(FightAudience, FightMessage)>();
    let rcv = MessageReceiver(fight_receiver);
    rcv.start(shared_game.clone());
    shared_game.lock().unwrap().set_fight_sender(fight_sender);

    let npcs = load_npcs()?;
    shared_game.init_npcs(npcs);

    for stream in listener.incoming() {
        let game_clone = shared_game.clone();

        let stream = stream?;

        let p = PlayerType::new_with_stream(stream);
        let uuid = p.uuid();


        {
            let mut game = match game_clone.lock() {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("error: {}", err);
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

    eprintln!("[{}]: player named", "SUCCESS".color(Green));
    eprintln!("[{}]: player id: {}", "SUCCESS".color(Green), p);
    loop {
        let s = get_player_command(p, g.clone())?;

        eprintln!("[{}]: Got player command", "SUCCESS".color(Green));
        {
            let mut g = match g.lock() {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("error: {}", err);
                    break;
                }
            };

            let resp = g.interpret(p, &s);
            eprintln!("[{}]: Got response", "SUCCESS".color(Green));
            match resp {
                Ok((aud, msg)) => {
                    eprintln!(
                        "[{}]: message: {:#?}",
                        "SUCCESS".color(Green),
                        msg.to_self().color(Magenta)
                    );
                    let results = g.send(&*aud, &*msg);
                    for (id, result) in results {
                        if let Err(e) = result {
                            eprintln!("[{}]: {:?}", "ERROR".color(Red), e);
                            let p = match g.remove_player(id) {
                                Some(p) => p,
                                None => Arc::new(Mutex::new(PlayerType::Human(Player::default()))),
                            };

                            std::mem::drop(p);
                        }
                    }
                }
                Err(e) => match e {
                    EnnuiError::Quit => {
                        g.remove_player(p);
                        break;
                    }
                    EnnuiError::Fatal(s) => eprintln!("[{}]: {:?}", "FATAL".color(Red), s),
                    e => eprintln!("[{}]: {:?}", "ERROR".color(Magenta), e),
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

    stream.write_all(b"enter your name: ")?;
    let name = stream.read_line()?;
    stream.write_all(b" > ")?;

    let mut g = g.lock().unwrap();
    let res = g
        .set_player_name(p, &name)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::NotFound));

    g.announce_player(p)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::NotFound))?;

    res
}


fn load_npcs() -> GameResult<Vec<PlayerType>> {
    let bytes = include_bytes!("../../data/npc.cbor");
    let v: Vec<YamlPlayer> = serde_cbor::from_slice(bytes)?;
    let mut ret: Vec<PlayerType> = vec![];

    for yp in v.into_iter() {
        ret.push(yp.into());
    }


    Ok(ret)
}
