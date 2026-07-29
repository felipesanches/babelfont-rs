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

/// Does this string look like a copyright notice rather than prose?
///
/// FontLab Studio 5 had a long-standing bug that copied the copyright notice
/// into the description field, and sources built with it still carry it.
pub(crate) fn is_copyright_notice(notice: &str) -> bool {
    let trimmed = notice.trim_start();
    let lowered = trimmed.to_lowercase();
    lowered.starts_with("copyright") || trimmed.starts_with('\u{a9}') || lowered.starts_with("(c)")
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
    fn test_is_copyright_notice() {
        // The forms actually seen in the affected sources.
        assert!(is_copyright_notice(
            "Copyright (c) 2011 by Gesine Todt. All rights reserved."
        ));
        assert!(is_copyright_notice("copyright 2012 vernon adams"));
        assert!(is_copyright_notice("(c) 2011 Kimberly Geswein"));
        assert!(is_copyright_notice("\u{a9} 2011 Kimberly Geswein"));

        // Leading whitespace must not hide it.
        assert!(is_copyright_notice("   Copyright 2011"));

        // Real prose in a description field is left alone, including text that
        // merely mentions copyright rather than being a notice.
        assert!(!is_copyright_notice(
            "A display face for headlines and posters."
        ));
        assert!(!is_copyright_notice(
            "This font is in the public domain; no copyright is claimed."
        ));
        assert!(!is_copyright_notice(""));
    }
}
