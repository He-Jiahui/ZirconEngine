use std::collections::{HashMap, HashSet};

use std::mem::size_of;
use unicode_script::UnicodeScript;

use crate::text::language::{TextCultureSelector, TextLanguageFallbackKey};
use crate::text::{CompositeFontDescriptor, FontFaceId, FontFamilyName, FontQuery, FontScript};

use super::database::FontDatabase;
use super::matching::{
    FontFamilyCandidateScope, ScopedFontFamilyCandidate, dedupe_scoped_families,
};

#[derive(Clone, Debug)]
struct CompiledSubFont {
    family: FontFamilyName,
    cultures: Option<Vec<TextCultureSelector>>,
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
                cultures: (!sub_font.cultures.is_empty()).then(|| {
                    sub_font
                        .cultures
                        .iter()
                        .filter_map(|culture| TextCultureSelector::compile(culture.as_str()))
                        .collect()
                }),
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
                            .as_ref()
                            .map(|cultures| {
                                cultures
                                    .len()
                                    .saturating_mul(size_of::<TextCultureSelector>())
                            })
                            .unwrap_or_default()
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

    /// Returns every culture-eligible family that a project composite can select.
    ///
    /// A line-metric envelope must bound future source text, so it cannot use the
    /// script/codepoint filter that narrows normal fallback resolution.
    pub(super) fn line_metric_envelope_families(
        &self,
        language: Option<TextLanguageFallbackKey>,
    ) -> Vec<FontFamilyName> {
        let mut families = self
            .sub_fonts
            .iter()
            .filter(|sub_font| culture_matches(sub_font.cultures.as_deref(), language))
            .map(|sub_font| sub_font.family.clone())
            .collect::<Vec<_>>();
        families.push(self.default_family.clone());
        families
    }

    fn matching_families(
        &self,
        script: FontScript,
        codepoints: &[char],
        language: Option<TextLanguageFallbackKey>,
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
        let mut priority_families = Vec::new();
        let mut generic_families = Vec::new();
        for (sub_font, matched) in self.sub_fonts.iter().zip(matched) {
            if !matched {
                continue;
            }
            match sub_font.cultures.as_deref() {
                None => generic_families.push(sub_font.family.clone()),
                Some(cultures) if culture_matches(Some(cultures), language) => {
                    priority_families.push(sub_font.family.clone());
                }
                Some(_) => {}
            }
        }
        priority_families.extend(generic_families);
        priority_families.push(self.default_family.clone());
        priority_families
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

pub(super) struct ClusterFallbackCandidates {
    pub(super) faces: Vec<FontFaceId>,
    pub(super) coverage_probe_count: usize,
}

pub(super) fn candidate_faces_for_cluster(
    database: &FontDatabase,
    query: &FontQuery,
    composite: Option<&CompositeFontIndex>,
    font_asset_owner: Option<&str>,
    script: FontScript,
    codepoints: &[char],
    language: Option<TextLanguageFallbackKey>,
) -> ClusterFallbackCandidates {
    let Some(first_codepoint) = codepoints.first().copied() else {
        return ClusterFallbackCandidates {
            faces: Vec::new(),
            coverage_probe_count: 0,
        };
    };
    let mut candidates = Vec::new();
    let mut coverage_probe_count = 0_usize;
    let mut seen = HashSet::new();
    let families = candidate_families(
        composite,
        query,
        database,
        font_asset_owner,
        script,
        codepoints,
        language,
    );
    database.record_fallback_family_visits(families.len());
    for candidate in families {
        let (family_candidates, family_coverage_probe_count) = match font_asset_owner {
            Some(owner) => database.font_asset_family_candidates_for_codepoint(
                owner,
                &candidate.family,
                query,
                first_codepoint,
                candidate.scope,
            ),
            None => {
                database.family_candidates_for_codepoint(&candidate.family, query, first_codepoint)
            }
        };
        coverage_probe_count = coverage_probe_count.saturating_add(family_coverage_probe_count);
        for face in family_candidates {
            if seen.insert(face) {
                candidates.push(face);
            }
        }
    }
    ClusterFallbackCandidates {
        faces: candidates,
        coverage_probe_count,
    }
}

fn candidate_families(
    composite: Option<&CompositeFontIndex>,
    query: &FontQuery,
    database: &FontDatabase,
    font_asset_owner: Option<&str>,
    script: FontScript,
    codepoints: &[char],
    language: Option<TextLanguageFallbackKey>,
) -> Vec<ScopedFontFamilyCandidate> {
    let external = FontFamilyCandidateScope::OwnerThenGlobal;
    let query_scope = if font_asset_owner.is_some() {
        FontFamilyCandidateScope::OwnerLocalOnly
    } else {
        external
    };
    let mut families = composite
        .map_or_else(Vec::new, |composite| {
            composite.matching_families(script, codepoints, language)
        })
        .into_iter()
        .map(|family| (family, external))
        .collect::<Vec<_>>();
    families.extend(
        query
            .families
            .iter()
            .cloned()
            .map(|family| (family, query_scope)),
    );
    if let Some(owner) = font_asset_owner {
        if let Some(asset_fallbacks) = database.font_asset_fallback_families(owner) {
            families.extend(
                asset_fallbacks
                    .iter()
                    .cloned()
                    .map(|family| (family, external)),
            );
        }
        families.extend(
            database
                .font_asset_base_fallback_families()
                .iter()
                .cloned()
                .map(|family| (family, external)),
        );
    } else {
        families.extend(
            database
                .fallback_families()
                .iter()
                .cloned()
                .map(|family| (family, external)),
        );
    }
    dedupe_scoped_families(families)
}

fn culture_matches(
    cultures: Option<&[TextCultureSelector]>,
    language: Option<TextLanguageFallbackKey>,
) -> bool {
    match cultures {
        None => true,
        Some(cultures) => language
            .is_some_and(|language| cultures.iter().any(|culture| culture.matches(language))),
    }
}

pub(super) fn script_for_char(codepoint: char) -> FontScript {
    FontScript::from_iso15924_tag(codepoint.script().short_name())
}
