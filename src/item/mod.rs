mod itemlist;

use self::ItemType::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum ItemType<T> {
    Weapon(T),
    Armor(T),
    Food(T),
    Drink(T),
    Container(Vec<ItemType<T>>),
    Inert(T),
}

impl ItemType<Item> {
    pub fn item(&self) -> Box<Item> {
        match &self {
            Weapon(ref t) | Armor(ref t) | Food(ref t) | Drink(ref t) | Inert(ref t) => {
                Box::new(t.clone())
            }
            _ => Box::new(Item {
                name: "".to_string(),
                description: "".to_string(),
                hook: "".to_string(),
            }),
        }
    }

    pub fn container(&self) -> Option<&Vec<ItemType<Item>>> {
        match &self {
            Container(ref t) => Some(t),
            _ => None,
        }
    }

    pub fn container_two(&self) -> Box<Vec<ItemType<Item>>> {
        match &self {
            Container(ref t) => Box::new(t.clone()),
            _ => Box::new(vec![]),
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
            Some(c) => c
                .iter()
                .map(|i| format!("\n - {}", i.item().name()))
                .collect::<String>(),
            None => self.item().description().to_string(),
        }
    }

    pub fn hook(&self) -> String {
        self.item().hook().clone()
    }

    pub fn transfer_item(
        &mut self,
        item_hook: &str,
        to: &mut ItemType<Item>,
    ) -> Result<String, String> {
        match (self, to) {
            (&mut Container(ref mut c), &mut Container(ref mut d)) => {
                let index = c
                    .iter()
                    .take_while(|&x| x.item().hook() != item_hook)
                    .count();

                if index == c.len() {
                    return Err(format!("you don't see a {}", item_hook));
                }

                let item = c.remove(index);
                if let Inert(ref i) = item {
                    return Err(format!("you can't take {}", item.item().name()));
                }
                d.push(item);

                Ok(format!(""))
            }
            _ => Err(format!("must be two containers")),
        }
    }

    pub fn find_by_hook(&self, item_hook: &str) -> Option<Box<Item>> {
        self.into_iter()
            .map(|i| i.item())
            .find(|x| x.hook() == item_hook)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    name: String,
    hook: String,
    description: String,
}

impl Item {
    pub fn new(n: &str, h: &str, d: &str) -> Self {
        let (name, hook, description) = (n.to_string(), h.to_string(), d.to_string());

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

impl IntoIterator for &ItemType<Item> {
    type Item = ItemType<Item>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Container(v) => v.to_vec().into_iter(),
            _ => vec![self.clone()].to_vec().into_iter(),
        }
    }
}
