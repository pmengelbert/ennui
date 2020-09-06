use std::io;
use std::collections::HashMap;

type CmdFunc = fn(&mut Player, &[&str]) -> String;

enum Status {
    Dead,
    Alive,
    Poisoned,
}

#[derive(Eq, PartialEq, Hash)]
enum MeterType {
    Hit,
    Mana,
    Movement,
}

pub struct Player {
    name: String,
    status: Vec<Status>,
    location: Room,
    meters: MeterGroup,
}

struct Item {
    name: String,
    description: String,
}

#[derive(Debug)]
struct Meter {
    current: isize,
    max: isize,
}

struct Room {
    name: String,
    description: String,
    items: HashMap<String, Item>,
}

impl Room {
    pub fn new(name: String, description: String) -> Room {
        let items = HashMap::new();
        Room { name, description, items }
    }

    pub fn to_string(&self) -> String {
        let mut builder = format!("\
            {}\n\
            ---------------------------------------\n\
            {}\n\
            ",
            self.name,
            self.description,
        );

        let items = self.items
            .values()
            .map(|v| format!(" - {}", v.name))
            .collect::<Vec<String>>()
            .join("\n");

        builder.push_str(&items);

        builder
    }
}

struct MeterGroup(HashMap<MeterType, Meter>);

#[test]
fn test_metergroup() {
    let mut mg = MeterGroup::new();
    mg.set(MeterType::Hit, (100, 100));

    mg.get(MeterType::Hit);
}

impl MeterGroup {
    pub fn new() -> MeterGroup {
        MeterGroup(HashMap::new())
    }

    pub fn set(&mut self, p: MeterType, points: (isize, isize)) {
        let h = &mut self.0;

        let (current, max) = points;
        h.insert(p, Meter { current, max });
    }

    pub fn get(&self, p: MeterType) -> Option<&Meter> {
        let h = &self.0;
        h.get(&p)
    }
}

impl Player {
    pub fn new(name: &str) -> Player {
        let mut mg = MeterGroup::new();

        let meters = [MeterType::Hit, MeterType::Mana, MeterType::Movement];
        
        mg.set(MeterType::Hit, (100, 100));
        mg.set(MeterType::Mana, (50, 50));
        mg.set(MeterType::Movement, (200, 200));

        let (room_name, description) = (
            "kitchen".to_string(),
            "you walk into the kitchen. it's dirty. \
             you stay here, but you want to leave.".to_string()
         );

        let mut room = Room::new(room_name, description);

        let item = Item { name: "a book".to_string(), description: "a nice book".to_string() };
        room.items.insert("book".to_string(), item);

        Player {
            name: String::from(name),
            status: vec![Status::Alive],
            meters: mg,
            location: room,
        }
    }
}

pub struct Interpreter {
    cmd: HashMap<String, CmdFunc>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut h: HashMap<String, CmdFunc> = HashMap::new();
        Interpreter {
            cmd: h,
        }
    }

    pub fn set(&mut self, name: &str, func: CmdFunc) {
        self.cmd.insert(name.to_string(), func);
    }

    fn get(&self, name: &str) -> Option<fn(&mut Player, &[&str]) -> String> {
        match self.cmd.get(name) {
            Some(&func) => Some(func),
            None => None,
        }
    }

    pub fn execute_string(&self, player: &mut Player, cmd: &str) -> String {
        let args: Vec<&str> = cmd
            .split_whitespace()
            .collect();

        let name = match args.get(0) {
            Some(s) => s,
            None => "",
        };

        self.execute_string_and_args(player, name, &args[1..])
    }

    fn execute_string_and_args(&self, player: &mut Player, name: &str, args: &[&str]) -> String {
        match self.get(name) {
            Some(func) => func(player, args),
            None => "i'll have to ask my lawyer about that".to_string(),
        }
    }
}

pub fn look(player: &mut Player, args: &[&str]) -> String {
    match args.len() {
        0 => player.location.to_string(),
        1 => {
            match player.location.items.get(args[0]) {
                Some(item) => item.description.clone(),
                None => format!("you don't see a {} here", args[0]),
            }
        }
        _ => String::from("you need to be specific. give me a one-word identification of the \
                          thing you want to look at. ok?")
    }
}

pub fn say(player: &mut Player, args: &[&str]) -> String {
    format!(r#"you say "{}""#, args.join(" "))
}

pub fn status(player: &mut Player, args: &[&str]) -> String {
    let (hit, mana, movement) = (
            player.meters.get(MeterType::Hit).unwrap(),
            player.meters.get(MeterType::Mana).unwrap(),
            player.meters.get(MeterType::Movement).unwrap(),
        );
    format!(r#"
            Hit Points: {} / {},
            Mana: {} / {},
            Movement: {} / {},
            "#,
            hit.current, hit.max,
            mana.current, mana.max,
            movement.current, movement.max)
}
