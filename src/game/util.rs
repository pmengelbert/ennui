use super::*;

pub fn random_insult() -> String {
    match rand::thread_rng().gen_range(1, 6) {
        1 => "dude wtf",
        2 => "i think you should leave",
        3 => "i'll have to ask my lawyer about that",
        4 => "that's ... uncommon",
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
