use crate::core::math::UVec2;

use super::{GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec};

pub(crate) const GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasResidentPage {
    spec: GlyphAtlasPageSpec,
    last_used_frame: u64,
    referenced_in_frame: bool,
}

impl GlyphAtlasResidentPage {
    #[cfg(test)]
    pub(crate) fn from_existing_page(spec: GlyphAtlasPageSpec) -> Self {
        Self {
            spec,
            last_used_frame: 0,
            referenced_in_frame: false,
        }
    }

    pub(crate) fn reserved(spec: GlyphAtlasPageSpec, frame_index: u64) -> Self {
        Self {
            spec,
            last_used_frame: frame_index,
            referenced_in_frame: true,
        }
    }

    pub(crate) fn key(&self) -> GlyphAtlasPageKey {
        self.spec.key
    }

    pub(crate) fn spec(&self) -> &GlyphAtlasPageSpec {
        &self.spec
    }

    pub(crate) fn replace_spec(&mut self, spec: GlyphAtlasPageSpec) {
        self.spec = spec;
    }

    pub(crate) fn mark_used(&mut self, frame_index: u64) {
        self.last_used_frame = frame_index;
        self.referenced_in_frame = true;
    }

    pub(crate) fn clear_frame_reference(&mut self) {
        self.referenced_in_frame = false;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasPageResidencyDecision {
    Allocate(GlyphAtlasPageKey),
    Evict(GlyphAtlasPageKey),
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasPageReservation {
    pub(crate) decision: GlyphAtlasPageResidencyDecision,
    pub(crate) page: Option<GlyphAtlasPageSpec>,
}

pub(crate) fn page_residency_decision(
    pages: &[GlyphAtlasResidentPage],
    format: GlyphAtlasFormat,
    max_pages_per_format: usize,
) -> GlyphAtlasPageResidencyDecision {
    if max_pages_per_format == 0 {
        return GlyphAtlasPageResidencyDecision::Blocked;
    }

    let pages_for_format = pages
        .iter()
        .filter(|page| page.key().format == format)
        .collect::<Vec<_>>();
    if pages_for_format.len() < max_pages_per_format {
        return GlyphAtlasPageResidencyDecision::Allocate(GlyphAtlasPageKey::new(
            format,
            next_free_page_index(&pages_for_format, format),
        ));
    }

    pages_for_format
        .iter()
        .filter(|page| !page.referenced_in_frame)
        .min_by(|left, right| {
            left.last_used_frame
                .cmp(&right.last_used_frame)
                .then_with(|| left.key().cmp(&right.key()))
        })
        .map(|page| GlyphAtlasPageResidencyDecision::Evict(page.key()))
        .unwrap_or(GlyphAtlasPageResidencyDecision::Blocked)
}

pub(crate) fn apply_page_residency_decision(
    pages: &mut Vec<GlyphAtlasResidentPage>,
    decision: GlyphAtlasPageResidencyDecision,
    page_size: UVec2,
    frame_index: u64,
) -> GlyphAtlasPageReservation {
    let page = match decision {
        GlyphAtlasPageResidencyDecision::Allocate(key)
        | GlyphAtlasPageResidencyDecision::Evict(key) => {
            Some(upsert_resident_page(pages, key, page_size, frame_index))
        }
        GlyphAtlasPageResidencyDecision::Blocked => None,
    };

    GlyphAtlasPageReservation { decision, page }
}

fn upsert_resident_page(
    pages: &mut Vec<GlyphAtlasResidentPage>,
    key: GlyphAtlasPageKey,
    page_size: UVec2,
    frame_index: u64,
) -> GlyphAtlasPageSpec {
    let spec = GlyphAtlasPageSpec::new(key, page_size);
    if let Some(existing) = pages.iter_mut().find(|page| page.key() == key) {
        existing.replace_spec(spec.clone());
        existing.mark_used(frame_index);
    } else {
        pages.push(GlyphAtlasResidentPage::reserved(spec.clone(), frame_index));
    }
    spec
}

fn next_free_page_index(pages: &[&GlyphAtlasResidentPage], format: GlyphAtlasFormat) -> u32 {
    let mut page_index = 0;
    while pages
        .iter()
        .any(|page| page.key() == GlyphAtlasPageKey::new(format, page_index))
    {
        page_index = page_index.saturating_add(1);
    }
    page_index
}

#[cfg(test)]
mod tests;
