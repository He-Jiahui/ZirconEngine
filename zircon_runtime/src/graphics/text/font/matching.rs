use crate::core::framework::render::{FontFamilyName, FontStretch, FontStyle, FontWeight};

use super::database::normalized_family_key;

pub(super) fn dedupe_families(
    families: impl IntoIterator<Item = FontFamilyName>,
) -> Vec<FontFamilyName> {
    let mut result: Vec<FontFamilyName> = Vec::new();
    for family in families {
        if family.is_empty() {
            continue;
        }
        let key = normalized_family_key(family.as_str());
        if result
            .iter()
            .any(|existing| normalized_family_key(existing.as_str()) == key)
        {
            continue;
        }
        result.push(family);
    }
    result
}

pub(super) fn weight_distance(candidate: FontWeight, requested: FontWeight) -> u16 {
    candidate.0.abs_diff(requested.0)
}

pub(super) fn stretch_distance(candidate: FontStretch, requested: FontStretch) -> u16 {
    candidate.0.abs_diff(requested.0)
}

pub(super) fn style_distance(candidate: FontStyle, requested: FontStyle) -> u8 {
    match (candidate, requested) {
        (FontStyle::Normal, FontStyle::Normal) => 0,
        (FontStyle::Italic, FontStyle::Italic) => 0,
        (FontStyle::Oblique(_), FontStyle::Oblique(_)) => 0,
        (FontStyle::Italic, FontStyle::Oblique(_)) | (FontStyle::Oblique(_), FontStyle::Italic) => {
            1
        }
        _ => 2,
    }
}
