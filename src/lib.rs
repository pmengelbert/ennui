use std::io;
use std::collections::HashMap;
enum Status {
    Dead,
    Alive,
    Poisoned,
}

#[derive(Eq, PartialEq, Hash)]
enum Point {
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

#[derive(Eq, PartialEq, Debug)]
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
}

struct MeterGroup(HashMap<Point, Meter>);

#[test]
fn test_metergroup() {
    let mut mg = MeterGroup::new();
    mg.set(Point::Hit, (100, 100));

    mg.get(Point::Hit);
}

impl MeterGroup {
    pub fn new() -> MeterGroup {
        MeterGroup(HashMap::new())
    }

    pub fn set(&mut self, p: Point, curmax: (isize, isize)) {
        let h = &mut self.0;

        let (current, max) = curmax;
        h.insert(p, Meter { current, max });
    }

    pub fn get(&self, p: Point) -> Option<&Meter> {
        let h = &self.0;
        h.get(&p)
    }
}

impl Player {
    pub fn new(name: &str) -> Player {
        let mut mg = MeterGroup::new();

        let meters = [Point::Hit, Point::Mana, Point::Movement];
        
        mg.set(Point::Hit, (100, 100));
        mg.set(Point::Mana, (50, 50));
        mg.set(Point::Movement, (200, 200));

        let mut room = Room::new("name".to_string(), r#"
 you walk into the kitchen. it's dirty.
 you stay here, but you want to leave.
 "#.to_string());

        room.items.insert("book".to_string(), Item { name: "book".to_string(), description: "a nice book".to_string() });

        Player {
            name: String::from(name),
            status: vec![Status::Alive],
            meters: mg,
            location: room,
        }
    }
}

pub struct Interpreter {
    cmd: HashMap<String, fn(&mut Player, &[String]) -> String>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut h: HashMap<String, fn(&mut Player, &[String]) -> String> = HashMap::new();
        Interpreter {
            cmd: h,
        }
    }

    pub fn set(&mut self, name: &str, func: fn(&mut Player, &[String]) -> String) {
        self.cmd.insert(name.to_string(), func);
    }

    pub fn get(&self, name: &str) -> Option<fn(&mut Player, &[String]) -> String> {
        match self.cmd.get(name) {
            Some(&func) => Some(func),
            None => None,
        }
    }

    pub fn execute_string(&self, player: &mut Player, cmd: &String) -> String {
        let args: Vec<String> = cmd
            .split_whitespace()
            .map(|x| x.to_string())
            .collect();

        let name = match args.get(0) {
            Some(s) => s,
            None => "",
        };

        self.execute_string_and_args(player, name, &args[1..])
    }

    fn execute_string_and_args(&self, player: &mut Player, name: &str, args: &[String]) -> String {
        match self.get(name) {
            Some(func) => func(player, args),
            None => "i'll have to ask my lawyer about that".to_string(),
        }
    }
}

pub fn look(player: &mut Player, args: &[String]) -> String {
    match args.len() {
        0 => player.location.description.clone(),
        1 => {
            match player.location.items.get(&args[0]) {
                Some(item) => item.description.clone(),
                None => format!("you don't see a {} here", args[0]),
            }
        }
        _ => String::from("you need to be specific. give me a one-word identification of the thing you want to look at. ok?")
    }
}

pub fn say(player: &mut Player, args: &[String]) -> String {
    format!(r#"you say "{}""#, args.join(" "))
}

pub fn status(player: &mut Player, args: &[String]) -> String {
    let (hit, mana, movement) = (
            player.meters.get(Point::Hit).unwrap(),
            player.meters.get(Point::Mana).unwrap(),
            player.meters.get(Point::Movement).unwrap(),
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
