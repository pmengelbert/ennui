mod itemlist;

use self::ItemType::*;

#[derive(Debug)]
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
                    .map(|i| format!("\n - {}", i.item().unwrap().name()))
                    .collect::<String>()
            },
            None => {
                self.item().unwrap().description().to_string()
            }
        }
    }

    pub fn transfer_item(&mut self, item_hook: &str, to: &mut ItemType<Item>) -> Result<String, String> {
        match (self, to) {
            (&mut Container(ref mut c),
                    &mut Container(ref mut d)) => {

                let index = c.iter()
                    .take_while(|&x| x.item().unwrap().hook() != item_hook)
                    .count();

                if index == c.len() {
                    return Err(format!("you don't see a {}", item_hook));
                }

                let item = c.remove(index);
                d.push(item);

                Ok(format!(""))
            },
            _ => Err(format!("must be two containers"))
        }
    }
}

#[derive(Debug)]
pub struct Item {
    name: String,
    hook: String,
    description: String,
}

impl Item {
    pub fn new(n: &str, h: &str, d: &str) -> Self {
        let (name, hook, description) = 
            (n.to_string(), h.to_string(), d.to_string());

        Self {
            name,
            hook,
            description,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn hook(&self) -> &String {
        &self.hook
    }

    pub fn description(&self) -> &String {
        &self.description
    }
}
