mod itemlist;

use self::ItemType::*;

pub enum ItemType<T> {
    Weapon(T),
    Armor(T),
    Food(T),
    Drink(T),
    Container(Vec<ItemType<T>>),
    Inert(T),
}

impl ItemType<Item> {
    pub fn item(&self) -> Option<&Item> {
        match &self {
            Weapon(ref t) | Armor(ref t) | Food(ref t) |
                Drink(ref t) | Inert(ref t) => Some(t),
            _ => None,
        }
    }

    pub fn container(&self) -> Option<&Vec<ItemType<Item>>> {
        match &self {
            Container(ref t) => Some(t),
            _ => None,
        }
    }

    pub fn container_mut(&mut self) -> Option<&mut Vec<ItemType<Item>>> {
        match self {
            &mut Container(ref mut t) => Some(t),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self.container() {
            Some(c) => {
                c.iter()
                    .map(|i| format!("\n - {}", i.item().unwrap().description()))
                    .collect::<String>()
            },
            None => {
                self.item().unwrap().description().to_string()
            }
        }
    }

    pub fn transfer_item(&mut self, item_name: &str, to: &mut ItemType<Item>) -> Result<String, String> {
        match (self, to) {
            (&mut Container(ref mut c),
                    &mut Container(ref mut d)) => {

                let index = c.iter()
                    .take_while(|&x| x.item().unwrap().name() != item_name)
                    .count();

                if index == c.len() {
                    return Err(format!("you don't see a {}", item_name));
                }

                let item = c.remove(index);
                d.push(item);

                Ok(format!(""))
            },
            _ => Err(format!("must be two containers"))
        }
    }
}

pub struct Item {
    name: String,
    description: String,
}

impl Item {
    pub fn new(n: &str, d: &str) -> Self {
        let (name, description) = (n.to_string(), d.to_string());

        Self {
            name,
            description,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &String {
        &self.description
    }
}
