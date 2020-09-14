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

pub struct Item {
    name: String,
    description: String,
}
