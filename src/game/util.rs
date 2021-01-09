use super::*;
use rand::{SeedableRng, random};

pub fn random_insult() -> String {
    let mut n = rand::rngs::mock::StepRng::new(48, 32895);
    let n: usize = n.gen::<usize>() % 5;
    match n {
        0 => "dude wtf",
        1 => "i think you should leave",
        2 => "i'll have to ask my lawyer about that",
        3 => "that's ... uncommon",
        _ => "that's an interesting theory... but will it hold up in the laboratory?",
    }
    .to_owned()
}

pub fn load_rooms(rooms: &mut RoomList) -> GameResult<()> {
    let bytes = include_bytes!("../../data/map.cbor");
    let v: Vec<Room> = serde_cbor::from_slice(bytes)?;

    for mut r in v {
        r.init();
        rooms.insert(r.loc(), r);
    }
    Ok(())
}
