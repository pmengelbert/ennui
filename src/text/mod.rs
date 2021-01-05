pub mod channel;
pub mod message;

pub enum Color {
    Red,
    #[allow(dead_code)]
    Green,
    #[allow(dead_code)]
    Yellow,
    #[allow(dead_code)]
    Blue,
    #[allow(dead_code)]
    Magenta,
    #[allow(dead_code)]
    Cyan,
    #[allow(dead_code)]
    White,
}

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
            let mut range_end = (&s[x..y]).rfind(' ').unwrap_or_else(|| s[x..y].len());
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

impl Color {
    fn to_code(&self) -> &str {
        match self {
            Color::Red => RED,
            Color::Green => GREEN,
            Color::Yellow => YELLOW,
            Color::Blue => BLUE,
            Color::Magenta => MAGENTA,
            Color::Cyan => CYAN,
            Color::White => WHITE,
        }
    }
}

const RED: &str = "[31m";
const GREEN: &str = "[32m";
const YELLOW: &str = "[33m";
const BLUE: &str = "[34m";
const MAGENTA: &str = "[35m";
const CYAN: &str = "[36m";
const WHITE: &str = "[37m";

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
