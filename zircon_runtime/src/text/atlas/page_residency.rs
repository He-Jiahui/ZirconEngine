//! 图集页的容量与逐帧引用策略：本帧已被绘制引用的页不能驱逐。
//! 普通分配优先增加页面，整页重建优先复用未引用页面；页键复用依靠世代变化拒绝旧上传与旧槽。

use crate::core::math::UVec2;

use super::{GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec};

pub(crate) const GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT: usize = 8;
const INLINE_PAGE_INDEX_CAPACITY: u32 = u128::BITS;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PageFormatOccupancy {
    page_count: usize,
    low_indices: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 记录页规格与当前帧保护状态；帧开始清除引用后，命中或分配路径再标记使用。
/// 使用时间用于跨帧 LRU，引用标志用于保护同一帧先前已规划的绘制。
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

    pub(crate) fn invalidate_contents(&mut self) {
        self.spec.generation = self.spec.generation.saturating_add(1);
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

    let occupancy = page_format_occupancy(pages, format);
    if occupancy.page_count < max_pages_per_format {
        return GlyphAtlasPageResidencyDecision::Allocate(GlyphAtlasPageKey::new(
            format,
            next_free_page_index(pages, format, occupancy),
        ));
    }

    least_recent_evictable_page(pages, format)
        .map(|page| GlyphAtlasPageResidencyDecision::Evict(page.key()))
        .unwrap_or(GlyphAtlasPageResidencyDecision::Blocked)
}

pub(crate) fn page_rebuild_residency_decision(
    pages: &[GlyphAtlasResidentPage],
    format: GlyphAtlasFormat,
    max_pages_per_format: usize,
) -> GlyphAtlasPageResidencyDecision {
    if max_pages_per_format == 0 {
        return GlyphAtlasPageResidencyDecision::Blocked;
    }

    if let Some(page) = least_recent_evictable_page(pages, format) {
        return GlyphAtlasPageResidencyDecision::Evict(page.key());
    }

    let occupancy = page_format_occupancy(pages, format);
    if occupancy.page_count < max_pages_per_format {
        return GlyphAtlasPageResidencyDecision::Allocate(GlyphAtlasPageKey::new(
            format,
            next_free_page_index(pages, format, occupancy),
        ));
    }

    GlyphAtlasPageResidencyDecision::Blocked
}

/// 应用刚从同一页面集合取得的驻留决策；调用方应先失效被驱逐页的槽和阴影。
/// 此函数只更新页规格及引用状态，不承担像素上传或缓存清理。
pub(crate) fn apply_page_residency_decision(
    pages: &mut Vec<GlyphAtlasResidentPage>,
    decision: GlyphAtlasPageResidencyDecision,
    page_size: UVec2,
    frame_index: u64,
) -> GlyphAtlasPageReservation {
    let page = match decision {
        GlyphAtlasPageResidencyDecision::Allocate(key)
        | GlyphAtlasPageResidencyDecision::Evict(key) => Some(upsert_resident_page(
            pages,
            decision,
            key,
            page_size,
            frame_index,
        )),
        GlyphAtlasPageResidencyDecision::Blocked => None,
    };

    GlyphAtlasPageReservation { decision, page }
}

fn upsert_resident_page(
    pages: &mut Vec<GlyphAtlasResidentPage>,
    decision: GlyphAtlasPageResidencyDecision,
    key: GlyphAtlasPageKey,
    page_size: UVec2,
    frame_index: u64,
) -> GlyphAtlasPageSpec {
    let generation = page_generation_for_reservation(pages, decision, key);
    let spec = GlyphAtlasPageSpec::new(key, page_size).with_generation(generation);
    if let Some(existing) = pages.iter_mut().find(|page| page.key() == key) {
        existing.replace_spec(spec.clone());
        existing.mark_used(frame_index);
    } else {
        pages.push(GlyphAtlasResidentPage::reserved(spec.clone(), frame_index));
    }
    spec
}

fn page_generation_for_reservation(
    pages: &[GlyphAtlasResidentPage],
    decision: GlyphAtlasPageResidencyDecision,
    key: GlyphAtlasPageKey,
) -> u64 {
    let current_generation = pages
        .iter()
        .find(|page| page.key() == key)
        .map(|page| page.spec().generation)
        .unwrap_or(0);

    match decision {
        GlyphAtlasPageResidencyDecision::Evict(_) => current_generation.saturating_add(1),
        GlyphAtlasPageResidencyDecision::Allocate(_) | GlyphAtlasPageResidencyDecision::Blocked => {
            current_generation
        }
    }
}

fn least_recent_evictable_page(
    pages: &[GlyphAtlasResidentPage],
    format: GlyphAtlasFormat,
) -> Option<&GlyphAtlasResidentPage> {
    pages
        .iter()
        .filter(|page| page.key().format == format && !page.referenced_in_frame)
        .min_by(|left, right| {
            left.last_used_frame
                .cmp(&right.last_used_frame)
                .then_with(|| left.key().cmp(&right.key()))
        })
}

fn page_format_occupancy(
    pages: &[GlyphAtlasResidentPage],
    format: GlyphAtlasFormat,
) -> PageFormatOccupancy {
    let mut occupancy = PageFormatOccupancy::default();
    for page in pages {
        let key = page.key();
        if key.format != format {
            continue;
        }
        occupancy.page_count += 1;
        if key.page_index < INLINE_PAGE_INDEX_CAPACITY {
            occupancy.low_indices |= 1_u128 << key.page_index;
        }
    }
    occupancy
}

fn next_free_page_index(
    pages: &[GlyphAtlasResidentPage],
    format: GlyphAtlasFormat,
    occupancy: PageFormatOccupancy,
) -> u32 {
    if occupancy.page_count < INLINE_PAGE_INDEX_CAPACITY as usize {
        return occupancy.low_indices.trailing_ones();
    }

    let mut occupied = vec![false; occupancy.page_count + 1];
    for page in pages {
        let key = page.key();
        if key.format == format {
            if let Some(slot) = usize::try_from(key.page_index)
                .ok()
                .and_then(|index| occupied.get_mut(index))
            {
                *slot = true;
            }
        }
    }
    occupied
        .iter()
        .position(|is_occupied| !*is_occupied)
        .and_then(|index| u32::try_from(index).ok())
        .unwrap_or(u32::MAX)
}

#[cfg(test)]
#[path = "page_residency/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "page_residency/tests/single_pass_index_tests.rs"]
mod single_pass_index_tests;
