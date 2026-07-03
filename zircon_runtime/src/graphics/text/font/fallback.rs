use crate::core::framework::render::{
    CompositeFontDescriptor, FontFaceId, FontFamilyName, FontQuery, FontScript, SubFontRange,
};

use super::database::FontDatabase;
use super::matching::dedupe_families;

pub(super) const DEFAULT_FALLBACK_MAX_DEPTH: u8 = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FallbackResolution {
    pub face: FontFaceId,
    pub missing: bool,
    pub source: FallbackResolutionSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FallbackResolutionSource {
    Primary,
    Fallback,
    LastResort,
    DepthLimitExceeded,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct MissingGlyphLog {
    entries: Vec<MissingGlyphDiagnostic>,
}

impl MissingGlyphLog {
    #[cfg(test)]
    pub(super) fn entries(&self) -> &[MissingGlyphDiagnostic] {
        &self.entries
    }

    fn push(&mut self, diagnostic: MissingGlyphDiagnostic) {
        self.entries.push(diagnostic);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MissingGlyphDiagnostic {
    pub script: FontScript,
    pub codepoints: Vec<u32>,
    pub reason: MissingGlyphReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MissingGlyphReason {
    MissingGlyph,
    DepthLimitExceeded,
}

pub(super) struct FallbackResolver<'a> {
    db: &'a FontDatabase,
    query: &'a FontQuery,
    composite: Option<&'a CompositeFontDescriptor>,
    max_depth: u8,
    diagnostics: MissingGlyphLog,
}

impl<'a> FallbackResolver<'a> {
    pub(super) fn new(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        composite: Option<&'a CompositeFontDescriptor>,
    ) -> Self {
        Self::with_max_depth(db, query, composite, DEFAULT_FALLBACK_MAX_DEPTH)
    }

    pub(super) fn with_max_depth(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        composite: Option<&'a CompositeFontDescriptor>,
        max_depth: u8,
    ) -> Self {
        Self {
            db,
            query,
            composite,
            max_depth,
            diagnostics: MissingGlyphLog::default(),
        }
    }

    #[cfg(test)]
    pub(super) fn diagnostics(&self) -> &MissingGlyphLog {
        &self.diagnostics
    }

    pub(super) fn resolve(
        &mut self,
        primary: FontFaceId,
        script: FontScript,
        codepoints: &[char],
    ) -> FallbackResolution {
        if codepoints.is_empty() || self.db.face_covers_all(primary, codepoints) {
            return FallbackResolution {
                face: primary,
                missing: false,
                source: FallbackResolutionSource::Primary,
            };
        }

        if self.max_depth == 0 {
            self.record_missing(script, codepoints, MissingGlyphReason::DepthLimitExceeded);
            return FallbackResolution {
                face: primary,
                missing: true,
                source: FallbackResolutionSource::DepthLimitExceeded,
            };
        }

        for candidate in self.candidates_for_cluster(script, codepoints) {
            if candidate != primary && self.db.face_covers_all(candidate, codepoints) {
                return FallbackResolution {
                    face: candidate,
                    missing: false,
                    source: FallbackResolutionSource::Fallback,
                };
            }
        }

        self.record_missing(script, codepoints, MissingGlyphReason::MissingGlyph);
        FallbackResolution {
            face: primary,
            missing: true,
            source: FallbackResolutionSource::LastResort,
        }
    }

    #[cfg(test)]
    pub(super) fn candidates_for_codepoint(&self, codepoint: char) -> Vec<FontFaceId> {
        self.candidates_for_cluster(script_for_char(codepoint), &[codepoint])
    }

    pub(super) fn resolve_codepoint(
        &mut self,
        primary: FontFaceId,
        codepoint: char,
    ) -> FallbackResolution {
        self.resolve(primary, script_for_char(codepoint), &[codepoint])
    }

    fn candidates_for_cluster(&self, script: FontScript, codepoints: &[char]) -> Vec<FontFaceId> {
        let Some(first_codepoint) = codepoints.first().copied() else {
            return Vec::new();
        };
        let mut candidates = Vec::new();
        for family in candidate_families(self.composite, self.query, self.db, script, codepoints) {
            for face in
                self.db
                    .family_candidates_for_codepoint(&family, self.query, first_codepoint)
            {
                if !candidates.contains(&face) {
                    candidates.push(face);
                }
            }
        }
        candidates
    }

    fn record_missing(
        &mut self,
        script: FontScript,
        codepoints: &[char],
        reason: MissingGlyphReason,
    ) {
        self.diagnostics.push(MissingGlyphDiagnostic {
            script,
            codepoints: codepoints
                .iter()
                .map(|codepoint| *codepoint as u32)
                .collect(),
            reason,
        });
    }
}

fn candidate_families(
    composite: Option<&CompositeFontDescriptor>,
    query: &FontQuery,
    database: &FontDatabase,
    script: FontScript,
    codepoints: &[char],
) -> Vec<FontFamilyName> {
    let mut families = Vec::new();
    if let Some(composite) = composite {
        for sub_font in &composite.sub_fonts {
            if sub_font_matches(script, codepoints, sub_font) {
                families.push(sub_font.family.clone());
            }
        }
        families.push(composite.default_family.clone());
    }
    families.extend(query.families.iter().cloned());
    families.extend(database.fallback_families().iter().cloned());
    dedupe_families(families)
}

fn sub_font_matches(script: FontScript, codepoints: &[char], sub_font: &SubFontRange) -> bool {
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
    script_match || range_match
}

fn script_for_char(codepoint: char) -> FontScript {
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

#[cfg(test)]
mod tests;
