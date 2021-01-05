use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Handle(pub Vec<String>);

impl PartialEq<&str> for Handle {
    fn eq(&self, other: &&str) -> bool {
        self.inner().iter().find(|&h| h == other).is_some()
    }
}

impl PartialEq<Handle> for &str {
    fn eq(&self, other: &Handle) -> bool {
        other.eq(self)
    }
}

impl PartialEq<&str> for &Handle {
    fn eq(&self, other: &&str) -> bool {
        self.inner().iter().find(|&h| h == other).is_some()
    }
}

impl PartialEq<&Handle> for &str {
    fn eq(&self, other: &&Handle) -> bool {
        other.eq(self)
    }
}

impl Handle {
    pub fn push(&mut self, s: String) {
        self.0.push(s);
    }

    fn inner(&self) -> &Vec<String> {
        &self.0
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
