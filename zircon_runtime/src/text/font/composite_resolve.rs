use std::collections::{HashMap, HashSet};
use std::mem::size_of;

use crate::text::{CompositeFontDescriptor, FontFaceId, FontFamilyName, FontQuery, FontScript};

use super::database::FontDatabase;
use super::matching::dedupe_families;

#[derive(Clone, Debug)]
struct CompiledSubFont {
    family: FontFamilyName,
    cultures: Vec<crate::text::FontCultureTag>,
}

#[derive(Clone, Copy, Debug)]
struct CompiledRange {
    start: u32,
    end: u32,
    sub_font_index: usize,
    prefix_max_end: u32,
}

/// Generation-owned CompositeFont lookup index. Script matches are direct and
/// Unicode interval lookup uses a binary upper bound plus prefix-max pruning,
/// while final projection preserves source declaration priority.
#[derive(Clone, Debug)]
pub(super) struct CompositeFontIndex {
    default_family: FontFamilyName,
    sub_fonts: Vec<CompiledSubFont>,
    scripts: HashMap<FontScript, Vec<usize>>,
    ranges: Vec<CompiledRange>,
    approximate_bytes: usize,
}

impl CompositeFontIndex {
    pub(super) fn compile(composite: &CompositeFontDescriptor) -> Self {
        let sub_fonts = composite
            .sub_fonts
            .iter()
            .map(|sub_font| CompiledSubFont {
                family: sub_font.family.clone(),
                cultures: sub_font.cultures.clone(),
            })
            .collect::<Vec<_>>();
        let mut scripts: HashMap<FontScript, Vec<usize>> = HashMap::new();
        let mut ranges = Vec::new();
        for (sub_font_index, sub_font) in composite.sub_fonts.iter().enumerate() {
            for script in &sub_font.scripts {
                scripts.entry(*script).or_default().push(sub_font_index);
            }
            for (start, end) in &sub_font.ranges {
                ranges.push(CompiledRange {
                    start: *start,
                    end: *end,
                    sub_font_index,
                    prefix_max_end: 0,
                });
            }
        }
        ranges.sort_by_key(|range| (range.start, range.sub_font_index, range.end));
        let mut prefix_max_end = 0;
        for range in &mut ranges {
            prefix_max_end = prefix_max_end.max(range.end);
            range.prefix_max_end = prefix_max_end;
        }
        let approximate_bytes = size_of::<Self>()
            + sub_fonts.len().saturating_mul(size_of::<CompiledSubFont>())
            + composite.default_family.as_str().len()
            + sub_fonts
                .iter()
                .map(|sub_font| {
                    sub_font.family.as_str().len()
                        + sub_font
                            .cultures
                            .iter()
                            .map(|culture| culture.as_str().len())
                            .sum::<usize>()
                })
                .sum::<usize>()
            + scripts
                .values()
                .map(|entries| entries.len().saturating_mul(size_of::<usize>()))
                .sum::<usize>()
            + ranges.len().saturating_mul(size_of::<CompiledRange>());
        Self {
            default_family: composite.default_family.clone(),
            sub_fonts,
            scripts,
            ranges,
            approximate_bytes,
        }
    }

    pub(super) const fn approximate_bytes(&self) -> usize {
        self.approximate_bytes
    }

    fn matching_families(
        &self,
        script: FontScript,
        codepoints: &[char],
        language: Option<&str>,
    ) -> Vec<FontFamilyName> {
        let mut matched = vec![false; self.sub_fonts.len()];
        if let Some(entries) = self.scripts.get(&script) {
            for sub_font_index in entries {
                matched[*sub_font_index] = true;
            }
        }
        for codepoint in codepoints {
            self.mark_range_matches(*codepoint as u32, &mut matched);
        }
        let mut families = self
            .sub_fonts
            .iter()
            .zip(matched)
            .filter(|(sub_font, matched)| *matched && culture_matches(&sub_font.cultures, language))
            .map(|(sub_font, _)| sub_font.family.clone())
            .collect::<Vec<_>>();
        families.push(self.default_family.clone());
        families
    }

    fn mark_range_matches(&self, codepoint: u32, matched: &mut [bool]) {
        let mut index = self
            .ranges
            .partition_point(|range| range.start <= codepoint);
        while index > 0 {
            index -= 1;
            let range = self.ranges[index];
            if range.prefix_max_end < codepoint {
                break;
            }
            if codepoint <= range.end {
                matched[range.sub_font_index] = true;
            }
        }
    }
}

pub(super) fn candidate_faces_for_cluster(
    database: &FontDatabase,
    query: &FontQuery,
    composite: Option<&CompositeFontIndex>,
    script: FontScript,
    codepoints: &[char],
    language: Option<&str>,
) -> Vec<FontFaceId> {
    let Some(first_codepoint) = codepoints.first().copied() else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();
    let families = candidate_families(composite, query, database, script, codepoints, language);
    database.record_fallback_family_visits(families.len());
    for family in families {
        for face in database.family_candidates_for_codepoint(&family, query, first_codepoint) {
            if seen.insert(face) {
                candidates.push(face);
            }
        }
    }
    candidates
}

fn candidate_families(
    composite: Option<&CompositeFontIndex>,
    query: &FontQuery,
    database: &FontDatabase,
    script: FontScript,
    codepoints: &[char],
    language: Option<&str>,
) -> Vec<FontFamilyName> {
    let mut families = composite.map_or_else(Vec::new, |composite| {
        composite.matching_families(script, codepoints, language)
    });
    families.extend(query.families.iter().cloned());
    families.extend(database.fallback_families().iter().cloned());
    dedupe_families(families)
}

fn culture_matches(cultures: &[crate::text::FontCultureTag], language: Option<&str>) -> bool {
    cultures.is_empty()
        || language.is_some_and(|language| cultures.iter().any(|culture| culture.matches(language)))
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
