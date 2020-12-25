use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Handle(pub Vec<String>);

impl Deref for Handle {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Handle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq<&str> for Handle {
    fn eq(&self, other: &&str) -> bool {
        self.iter().find(|h| h == other).is_some()
    }
}

impl PartialEq<Handle> for &str {
    fn eq(&self, other: &Handle) -> bool {
        other.eq(self)
    }
}

impl PartialEq<&str> for &Handle {
    fn eq(&self, other: &&str) -> bool {
        self.iter().find(|h| h == other).is_some()
    }
}

impl PartialEq<&Handle> for &str {
    fn eq(&self, other: &&Handle) -> bool {
        other.eq(self)
    }
}

#[macro_export]
macro_rules! handle {
    ( $( $name:ident ),* ) => {
        {
            let mut h: Handle = Handle(Vec::new());
            $( h.push(stringify!($name).to_owned()); )*
            h
        }
    }
}

#[cfg(test)]
mod handle_test {
    use super::*;

    #[test]
    fn handle_test() {
        let h = handle![sword, rusty];
        println!("{:#?}", h);
        assert_eq!("sword", h);
        assert_eq!("rusty", h);
    }
}
