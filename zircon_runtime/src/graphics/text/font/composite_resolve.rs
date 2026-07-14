use crate::core::framework::render::{
    CompositeFontDescriptor, FontFaceId, FontFamilyName, FontQuery, FontScript, SubFontRange,
};

use super::database::FontDatabase;
use super::matching::dedupe_families;

pub(super) fn candidate_faces_for_cluster(
    database: &FontDatabase,
    query: &FontQuery,
    composite: Option<&CompositeFontDescriptor>,
    script: FontScript,
    codepoints: &[char],
    language: Option<&str>,
) -> Vec<FontFaceId> {
    let Some(first_codepoint) = codepoints.first().copied() else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    for family in candidate_families(composite, query, database, script, codepoints, language) {
        for face in database.family_candidates_for_codepoint(&family, query, first_codepoint) {
            if !candidates.contains(&face) {
                candidates.push(face);
            }
        }
    }
    candidates
}

fn candidate_families(
    composite: Option<&CompositeFontDescriptor>,
    query: &FontQuery,
    database: &FontDatabase,
    script: FontScript,
    codepoints: &[char],
    language: Option<&str>,
) -> Vec<FontFamilyName> {
    let mut families = Vec::new();
    if let Some(composite) = composite {
        for sub_font in &composite.sub_fonts {
            if sub_font_matches(script, codepoints, language, sub_font) {
                families.push(sub_font.family.clone());
            }
        }
        families.push(composite.default_family.clone());
    }
    families.extend(query.families.iter().cloned());
    families.extend(database.fallback_families().iter().cloned());
    dedupe_families(families)
}

fn sub_font_matches(
    script: FontScript,
    codepoints: &[char],
    language: Option<&str>,
    sub_font: &SubFontRange,
) -> bool {
    let script_match = !sub_font.scripts.is_empty()
        && sub_font
            .scripts
            .iter()
            .any(|sub_font_script| *sub_font_script == script);
    let range_match = !sub_font.ranges.is_empty()
        && codepoints.iter().any(|codepoint| {
            let codepoint = *codepoint as u32;
            sub_font
                .ranges
                .iter()
                .any(|(start, end)| *start <= codepoint && codepoint <= *end)
        });
    let culture_match = sub_font.cultures.is_empty()
        || language.is_some_and(|language| {
            sub_font
                .cultures
                .iter()
                .any(|culture| culture.matches(language))
        });
    (script_match || range_match) && culture_match
}

pub(super) fn script_for_char(codepoint: char) -> FontScript {
    match codepoint as u32 {
        0x0041..=0x007A | 0x00C0..=0x024F | 0x1E00..=0x1EFF => FontScript::Latin,
        0x0370..=0x03FF | 0x1F00..=0x1FFF => FontScript::Greek,
        0x0400..=0x052F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F => FontScript::Cyrillic,
        0x0590..=0x05FF => FontScript::Hebrew,
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF => FontScript::Arabic,
        0x0900..=0x097F => FontScript::Devanagari,
        0x3040..=0x309F => FontScript::Hiragana,
        0x30A0..=0x30FF | 0x31F0..=0x31FF => FontScript::Katakana,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF => FontScript::Han,
        0xAC00..=0xD7AF | 0x1100..=0x11FF | 0x3130..=0x318F => FontScript::Hangul,
        other => FontScript::Other(other),
    }
}
