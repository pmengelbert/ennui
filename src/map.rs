
pub mod map {
    use crate::*;
    use std::collections::HashMap;

    #[derive(Eq, PartialEq, Hash)]
    pub struct Coord(pub i32, pub i32);
    impl Coord {
        pub fn north(&self) -> Coord {
            Coord(self.0, self.1 + 1)
        }

        pub fn south(&self) -> Coord {
            Coord(self.0, self.1 - 1)
        }
    }

    pub struct Map {
        pub map: HashMap<Coord, Room>
    }
}
