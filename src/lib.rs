use std::ops::{DerefMut, Deref};
use std::fmt;
use std::collections::HashMap;
use std::process;
use rand::Rng;

type CmdFunc = fn(&mut Player, &[&str]) -> String;
struct ItemList(HashMap<String, Item>);

impl Deref for ItemList {
    type Target = HashMap<String, Item>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for ItemList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let item_string = match self.len() {
            0 => "".to_string(),
            _ => self
                .values()
                .map(|v| format!("\n - {}", v.name))
                .collect::<String>()

        };

        write!(f, "{}", item_string)
    }
}

impl Item {
    pub fn wearable(&self) -> bool {
        self.kind == ItemType::Wearable
    }
}

#[derive(PartialEq, Eq)]
enum ItemType {
    Wearable,
    Edible,
    Normal,
}

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
    items: ItemList,
    clothing: ItemList,
}

struct Item {
    kind: ItemType,
    name: String,
    description: String,
}

struct Meter {
    current: isize,
    max: isize,
}

struct Room {
    name: String,
    description: String,
    items: ItemList,
}

impl Room {
    pub fn new(name: String, description: String) -> Room {
        let items = ItemList(HashMap::new());
        Room { name, description, items }
    }

    pub fn to_string(&self) -> String {
        let mut builder = format!("\
            {}\n\
            ---------------------------------------\n\
            {}{}",
            self.name,
            self.description,
            self.items,
        );

        builder
    }
}

struct MeterGroup(HashMap<MeterType, Meter>);

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

        let item = Item {
            kind: ItemType::Normal,
            name: "a book".to_string(),
            description: "a nice book".to_string()
        };
        room.items.insert("book".to_string(), item);

        let item = Item {
            kind: ItemType::Wearable,
            name: "a shirt".to_string(),
            description: "a nice book".to_string()
        };
        room.items.insert("shirt".to_string(), item);

        Player {
            name: String::from(name),
            status: vec![Status::Alive],
            meters: mg,
            location: room,
            items: ItemList(HashMap::new()),
            clothing: ItemList(HashMap::new()),
        }
    }

    pub fn take(&mut self, item_name: &str) -> Result<String, String> {
        let item = match self.location.items.remove(item_name) {
            Some(item) => item,
            None => { return Err(item_name.to_string()); }
        };

        self.items.insert(item_name.to_string(), item);
        Ok(item_name.to_string())
    }

    pub fn drop(&mut self, item_name: &str) -> Result<String, String> {
        let item = match self.items.remove(item_name) {
            Some(item) => item,
            None => { return Err(item_name.to_string()); }
        };

        self.location.items.insert(item_name.to_string(), item);
        Ok(item_name.to_string())
    }

    pub fn wear(&mut self, item_name: &str) -> Result<(), String> {
        match self.items.get(item_name) {
            Some(i) => {
                if !i.wearable() { return Err("you can't wear that!".to_string()); }

                let item = self.items.remove(item_name).unwrap();
                self.clothing.insert(item_name.to_string(), item);
                Ok(())
            },
            None => { return Err(format!("you're not holding a {}", item_name)); },
        }


    }

    pub fn remove(&mut self, item_name: &str) -> Result<(), String> {
        let (clothing, items) = (&mut self.clothing, &mut self.items);

        match clothing.get(item_name) {
            Some(i) => {
                let item = clothing.remove(item_name).unwrap();
                items.insert(item_name.to_string(), item);
                Ok(())
            },
            None => { return Err(format!("you're not holding a {}", item_name)); },
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

    fn get(&self, name: &str) -> Option<CmdFunc> {
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
            None => random_insult(),
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

pub fn take(player: &mut Player, args: &[&str]) -> String {
    match args.len() {
        0 => "what do you want to take?".to_string(),
        1 => match player.take(args[0]) {
            Ok(item) => format!("you take the {}", item),
            Err(err) => format!("you don't see a {} here", err),
        }
        _ => "i'm not sure what you want to take".to_string()
    }
}

pub fn inventory(player: &mut Player, args: &[&str]) -> String {
     match player.items.len() {
         0 => "you don't own anything".to_string(),
         _ => {
             format!("you have the following items:{}", player.items)
         }
    }
}

pub fn drop(player: &mut Player, args: &[&str]) -> String {
    match args.len() {
        0 => "drop what now?".to_string(),
        1 => match player.drop(args[0]) {
            Ok(item) => format!("you drop the {}", item),
            Err(item) => format!("you don't have a {} to drop", item),
        },
        _ => "you're confusing me!".to_string(),
    }
}

pub fn random_insult() -> String {
    match rand::thread_rng().gen_range(1, 6) {
        1 => "dude wtf".to_string(),
        2 => "i think you should leave".to_string(),
        3 => "i'll have to ask my lawyer about that".to_string(),
        4 => "that's ... uncommon".to_string(),
        _ => "that's an interesting theory... but will it hold up in the laboratory?".to_string()
    }
}

pub fn quit(player: &mut Player, args: &[&str]) -> String {
    println!("goodbye");
    std::process::exit(0);
}

pub fn wear(player: &mut Player, args: &[&str]) -> String {
    match args.len() {
        0 => "what do you want to wear?".to_string(),
        1 => match player.wear(args[0]) {
            Ok(_) => format!("you put on the {}", args[0]),
            Err(msg) => msg,
        },
        _ => "you really need to pick one thing".to_string(),
    }
}

pub fn remove(player: &mut Player, args: &[&str]) -> String {
    match args.len() {
        0 => "while I won't complain that you want to take off \
            your clothes, you need to tell me what you want to take off first.".to_string(),
        1 => match player.remove(args[0]) {
            Ok(_) => format!("you take off the {}", args[0]),
            Err(msg) => msg,
        },
        _ => "you really need to pick one thing".to_string(),
    }
}
