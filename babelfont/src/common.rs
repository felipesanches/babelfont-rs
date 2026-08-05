use std::str::FromStr;

use crate::axis::Tag;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// Useful font-related constants
pub mod constants;
pub(crate) mod decomposition;
pub(crate) mod formatspecific;
mod node;
pub(crate) mod otvalue;
pub use node::{Node, NodeType};

use crate::BabelfontError;
pub use formatspecific::FormatSpecific;
pub use otvalue::CustomOTValues;

/// Flatten a string so it can be stored in a `name` record.
///
/// Name records are single-line. Sources routinely carry hard line breaks in
/// the license description and copyright -- an SFD encodes them in the
/// `LangName` line, FontLab stores them with carriage returns -- and passing
/// those through produces a font that fails QA and renders the break as a
/// stray glyph in some tools.
///
/// Any run of line breaks, with the whitespace around it, becomes one space.
pub(crate) fn single_line(value: &str) -> String {
    value
        .split(['\r', '\n'])
        // A name record cannot carry a control character at all. Line breaks
        // become the join below; anything else in C0 (NUL especially) is simply
        // removed. Two families in a Google Fonts corpus -- architectsdaughter
        // and dawningofanewday -- encode a literal U+0000 in their licence text
        // (`+AA0ACgAA-` is CR, LF, NUL), and it reached the built font.
        .map(|part| part.chars().filter(|c| !c.is_control()).collect::<String>())
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn tag_from_string(s: &str) -> Result<Tag, BabelfontError> {
    if s.len() > 4 {
        return Err(BabelfontError::General(format!(
            "Tag must be 4 characters or less, got: '{}'",
            s
        )));
    }
    let mut chars = s.bytes().collect::<Vec<u8>>();
    while chars.len() < 4 {
        chars.push(b' ');
    }
    Ok(Tag::new(&chars[0..4].try_into().map_err(|_| {
        BabelfontError::General(format!("Bad tag: '{}'", s))
    })?))
}
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare]
/// A position in 2D space, with an optional angle
pub struct Position {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Angle in degrees
    #[serde(default, skip_serializing_if = "crate::serde_helpers::is_zero")]
    pub angle: f32,
}

impl Position {
    /// Create a zeroed Position
    pub fn zero() -> Position {
        Position {
            x: 0.0,
            y: 0.0,
            angle: 0.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare]
pub struct Color {
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: i32,
}

#[cfg(feature = "ufo")]
mod ufo {
    use super::*;
    impl From<&norad::Color> for Color {
        fn from(c: &norad::Color) -> Self {
            let (red, green, blue, alpha) = c.channels();
            Color {
                r: (red * 255.0) as i32,
                g: (green * 255.0) as i32,
                b: (blue * 255.0) as i32,
                a: (alpha * 255.0) as i32,
            }
        }
    }
    impl TryFrom<&Color> for norad::Color {
        type Error = BabelfontError;
        fn try_from(c: &Color) -> Result<Self, BabelfontError> {
            Ok(norad::Color::new(
                c.r as f64 / 255.0,
                c.g as f64 / 255.0,
                c.b as f64 / 255.0,
                c.a as f64 / 255.0,
            )?)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
#[typeshare]
/// Direction of text flow
pub enum Direction {
    /// Left to right text flow
    LeftToRight,
    /// Right to left text flow
    RightToLeft,
    /// Top to bottom text flow
    TopToBottom,
    /// Bidirectional,
    Bidi,
}

impl FromStr for Direction {
    type Err = BabelfontError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lefttoright" | "ltr" => Ok(Direction::LeftToRight),
            "righttoleft" | "rtl" => Ok(Direction::RightToLeft),
            "toptobottom" | "ttb" | "vtr" => Ok(Direction::TopToBottom),
            "bidi" => Ok(Direction::Bidi),
            _ => Err(BabelfontError::General(format!(
                "Invalid direction string: {}",
                s
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line() {
        // The case this exists for: an OFL description with hard breaks. An
        // SFD encodes these as carriage returns in its LangName line.
        assert_eq!(
            single_line(
                "This Font Software is licensed under the SIL Open Font License,\rVersion 1.1."
            ),
            "This Font Software is licensed under the SIL Open Font License, Version 1.1."
        );

        // Every flavour of break, and runs of them, collapse to one space.
        assert_eq!(single_line("a\nb"), "a b");
        assert_eq!(single_line("a\r\nb"), "a b");
        assert_eq!(single_line("a\n\n\nb"), "a b");

        // Whitespace around a break is absorbed rather than doubled up.
        assert_eq!(single_line("a  \n  b"), "a b");

        // Leading and trailing breaks leave no stray space.
        assert_eq!(single_line("\na\n"), "a");

        // A string with no break is returned unchanged, including interior
        // spacing, which is not ours to normalise.
        assert_eq!(single_line("a  b"), "a  b");
        assert_eq!(single_line(""), "");

        // A control character can never appear in a name record. NUL is the
        // one that actually occurs: two families encode U+0000 in their
        // licence text.
        assert_eq!(single_line("a\u{0}b"), "ab");
        assert_eq!(
            single_line("---\r\n\u{0}SIL OPEN FONT"),
            "--- SIL OPEN FONT"
        );
        assert_eq!(single_line("\u{0}"), "");
        assert_eq!(single_line("a\u{7}\u{1b}b"), "ab");
        // A part that is only control characters must not leave a stray space.
        assert_eq!(single_line("a\n\u{0}\nb"), "a b");

        // Idempotent: flattening an already-flat string changes nothing.
        let once = single_line("a\nb\nc");
        assert_eq!(single_line(&once), once);
    }
}
