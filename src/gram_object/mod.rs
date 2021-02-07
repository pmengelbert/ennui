use serde::{Deserialize, Serialize};
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Hook(pub Vec<String>);

pub struct Grabber<'a> {
    pub handle: &'a str,
    pub index: usize,
}

impl<'a> Grabber<'a> {
    pub fn from_str(handle: &'a str) -> Self {
        let num = match handle.find('.') {
            Some(n) => n,
            None => return Self { handle, index: 0 },
        };

        match &handle[0..num] {
            "all" => todo!(),
            s => match s.parse::<usize>() {
                Ok(n) if n > 0 => Self {
                    handle: &handle[num + 1..],
                    index: n - 1,
                },
                _ => Self {
                    handle: &handle[num + 1..],
                    index: 0,
                },
            },
        }
    }
}

impl<'a> From<&'a str> for Grabber<'a> {
    fn from(s: &'a str) -> Self {
        Grabber::from_str(s)
    }
}

impl PartialEq<&str> for Hook {
    fn eq(&self, other: &&str) -> bool {
        self.inner().iter().any(|h| h == other)
    }
}

impl PartialEq<Hook> for &str {
    fn eq(&self, other: &Hook) -> bool {
        other.eq(self)
    }
}

impl PartialEq<&str> for &Hook {
    fn eq(&self, other: &&str) -> bool {
        self.inner().iter().any(|h| h == other)
    }
}

impl PartialEq<&Hook> for &str {
    fn eq(&self, other: &&Hook) -> bool {
        other.eq(self)
    }
}

impl Hook {
    pub fn push(&mut self, s: String) {
        self.0.push(s);
    }

    fn inner(&self) -> &Vec<String> {
        &self.0
    }
}
