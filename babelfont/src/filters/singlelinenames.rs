//! Flatten every name-table string to a single line.
//!
//! A `name` record is single-line, but source formats are not restricted to
//! single-line values: an SFD encodes hard line breaks (and sometimes NULs)
//! into its `LangName` licence text, FontLab stores carriage returns, and a
//! Glyphs file may carry line breaks in its properties. A faithful conversion
//! keeps them, so this is an opt-in correction rather than convertor
//! behaviour: applying it produces a font that differs from the source's
//! stated text.
//!
//! Any run of line breaks, with the whitespace around it, becomes one space;
//! every other control character is removed.

use crate::{common::single_line, filters::FontFilter, I18NDictionary};

/// Flatten every name-table string to a single line.
#[derive(Default)]
pub struct SingleLineNames;

impl SingleLineNames {
    /// Create a new SingleLineNames filter
    pub fn new() -> Self {
        SingleLineNames
    }
}

fn flatten(dict: &mut I18NDictionary) {
    for value in dict.0.values_mut() {
        let flat = single_line(value);
        if flat != *value {
            *value = flat;
        }
    }
}

impl FontFilter for SingleLineNames {
    fn apply(&self, font: &mut crate::Font) -> Result<(), crate::BabelfontError> {
        let names = &mut font.names;
        for dict in [
            &mut names.copyright,
            &mut names.family_name,
            &mut names.preferred_subfamily_name,
            &mut names.unique_id,
            &mut names.full_name,
            &mut names.version,
            &mut names.postscript_name,
            &mut names.trademark,
            &mut names.manufacturer,
            &mut names.designer,
            &mut names.description,
            &mut names.manufacturer_url,
            &mut names.designer_url,
            &mut names.license,
            &mut names.license_url,
            &mut names.typographic_family,
            &mut names.typographic_subfamily,
            &mut names.compatible_full_name,
            &mut names.sample_text,
            &mut names.postscript_cid_name,
            &mut names.wws_family_name,
            &mut names.wws_subfamily_name,
            &mut names.variations_postscript_name_prefix,
        ] {
            flatten(dict);
        }
        Ok(())
    }

    fn from_str(_s: &str) -> Result<Self, crate::BabelfontError>
    where
        Self: Sized,
    {
        Ok(SingleLineNames::new())
    }

    #[cfg(feature = "cli")]
    fn arg() -> clap::Arg
    where
        Self: Sized,
    {
        clap::Arg::new("singlelinenames")
            .long("single-line-names")
            .help(
                "Flatten every name-table string to a single line and strip \
                 control characters. A correction, not a faithful conversion: \
                 a source may deliberately carry line breaks in its licence or \
                 description text.",
            )
            .action(clap::ArgAction::SetTrue)
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_every_field_and_language() {
        let mut font = crate::Font::new();
        font.names.license.set_default(
            "This Font Software is licensed under the SIL Open Font License,\rVersion 1.1."
                .to_string(),
        );
        font.names
            .description
            .0
            .insert("fr".to_string(), "Une\ndescription".to_string());
        // A NUL encoded into licence text must go too.
        font.names
            .copyright
            .set_default("Copyright\u{0} 2011".to_string());

        SingleLineNames::new().apply(&mut font).unwrap();

        assert_eq!(
            font.names.license.get_default().unwrap(),
            "This Font Software is licensed under the SIL Open Font License, Version 1.1."
        );
        assert_eq!(font.names.description.0.get("fr").unwrap(), "Une description");
        assert_eq!(font.names.copyright.get_default().unwrap(), "Copyright 2011");
    }

    #[test]
    fn leaves_single_line_values_untouched() {
        let mut font = crate::Font::new();
        font.names.family_name.set_default("Two  Words".to_string());
        SingleLineNames::new().apply(&mut font).unwrap();
        // Interior spacing is not ours to normalise.
        assert_eq!(font.names.family_name.get_default().unwrap(), "Two  Words");
    }
}
