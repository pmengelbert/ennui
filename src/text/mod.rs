pub mod message;
pub mod channel;

use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum Color {
    Red(String),
    #[allow(dead_code)]
    Green(String),
    #[allow(dead_code)]
    Yellow(String),
    #[allow(dead_code)]
    Blue(String),
    #[allow(dead_code)]
    Magenta(String),
    #[allow(dead_code)]
    Cyan(String),
    #[allow(dead_code)]
    White(String),
}

use std::fmt;
use Color::*;

pub trait Wrap {
    fn wrap(&self, line_length: usize) -> String;
}

impl<T> Wrap for T
where
    T: AsRef<str>,
{
    fn wrap(&self, line_length: usize) -> String {
        use std::cmp::min;

        let s = self.as_ref();

        let mut x = 0_usize;
        let mut y = min(s.len(), line_length);
        let mut ret = String::with_capacity(s.len() + s.len() / line_length);

        while x <= y && y < s.len() {
            let mut range_end = (&s[x..y]).rfind(' ').unwrap_or(s[x..y].len());
            let newline = (&s[x..y]).rfind('\n');
            if let Some(n) = newline {
                range_end = n;
            }

            y = x + range_end;

            ret.push_str(&s[x..y]);
            ret.push('\n');

            x = y + 1;
            y = min(s.len(), x + line_length);
        }
        ret.push_str(&s[x..s.len()]);

        ret
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let code = self.to_code();

        write!(f, "\u{001b}{}{}\u{001b}[37m", code, self.safe_unwrap())
    }
}

impl Color {
    fn to_code(&self) -> &str {
        match self {
            Red(_) => RED,
            Green(_) => GREEN,
            Yellow(_) => YELLOW,
            Blue(_) => BLUE,
            Magenta(_) => MAGENTA,
            Cyan(_) => CYAN,
            White(_) => WHITE,
        }
    }

    fn safe_unwrap(&self) -> &str {
        match self {
            Red(s) | Green(s) | Yellow(s) | Blue(s) | Magenta(s) | Cyan(s) | White(s) => s,
        }
    }
}

const RED: &'static str = "[31m";
const GREEN: &'static str = "[32m";
const YELLOW: &'static str = "[33m";
const BLUE: &'static str = "[34m";
const MAGENTA: &'static str = "[35m";
const CYAN: &'static str = "[36m";
const WHITE: &'static str = "[37m";

#[cfg(test)]
mod text_test {
    use super::Wrap;

    const DESC: &'static str = "You are at the Temple Yard of Dragonia. Beautiful marble stairs lead \
    up to the Temple of Dragonia. You feel small as you stare up the huge pillars making the entrance \
    to the temple. This place serves as a sanctuary where the  people of the city can come and seek \
    refuge, and rest their tired bones. Just north of here is the common square, and the temple opens \
    to the south.";

    #[test]
    fn test_line_wrap() {
        assert_eq!("abcd".wrap(7), "abcd");
        assert_eq!(
            "1 3 5 7 9 1 39234 290 290 5 7 9".wrap(10),
            "1 3 5 7 9\n1 39234\n290 290 5\n7 9"
        );

        assert_eq!(
            "1 3 5 7 9 1 39234 290 290 5 7 9".wrap(10),
            "1 3 5 7 9\n1 39234\n290 290 5\n7 9"
        );
    }

    #[test]
    fn test_line_wrap_idempotency() {
        let desc = DESC.wrap(80);
        let second = desc.wrap(80);
        assert_eq!(desc, second.wrap(80));
        assert_ne!(desc, second.wrap(75));
    }
}

pub fn article(noun: &str) -> String {
    let suffix = match noun.to_lowercase().chars().next().unwrap_or('\0') {
        'a' | 'e' | 'i' | 'o' | 'u' => "n",
        _ => "",
    };

    format!("a{} {}", suffix, noun)
}
