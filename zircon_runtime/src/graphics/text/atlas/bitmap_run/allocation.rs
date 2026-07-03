use std::collections::BTreeMap;

use crate::core::math::UVec2;

use super::super::{
    GlyphAtlasAllocation, GlyphAtlasDirtyPage, GlyphAtlasFormat, GlyphAtlasPageKey,
    GlyphAtlasPageResidencyDecision, GlyphAtlasRect, GlyphAtlasShelfAllocator,
};
use super::failure::GlyphAtlasBitmapAllocationFailureReason;
use super::types::{GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSource};

pub(super) fn allocate_bitmap_source(
    plan: &mut GlyphAtlasBitmapRunPlan,
    allocators: &mut BTreeMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>,
    active_pages: &mut BTreeMap<GlyphAtlasFormat, GlyphAtlasPageKey>,
    source: GlyphAtlasBitmapSource,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
) -> Result<GlyphAtlasAllocation, GlyphAtlasBitmapAllocationFailureReason> {
    if source.content_size.x > page_size.x || source.content_size.y > page_size.y {
        return Err(GlyphAtlasBitmapAllocationFailureReason::OversizedGlyph);
    }

    let page_key = ensure_active_bitmap_page(
        plan,
        allocators,
        active_pages,
        source.format,
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
    )?;

    if let Some(allocation) =
        allocate_without_mutating_on_failure(allocators, page_key, source.content_size)
    {
        return Ok(allocation);
    }

    let page_key = reserve_bitmap_page(
        plan,
        allocators,
        active_pages,
        source.format,
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
    )?;

    allocate_without_mutating_on_failure(allocators, page_key, source.content_size)
        .ok_or(GlyphAtlasBitmapAllocationFailureReason::OversizedGlyph)
}

fn ensure_active_bitmap_page(
    plan: &mut GlyphAtlasBitmapRunPlan,
    allocators: &mut BTreeMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>,
    active_pages: &mut BTreeMap<GlyphAtlasFormat, GlyphAtlasPageKey>,
    format: GlyphAtlasFormat,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
) -> Result<GlyphAtlasPageKey, GlyphAtlasBitmapAllocationFailureReason> {
    if let Some(page_key) = active_pages.get(&format).copied() {
        return Ok(page_key);
    }

    reserve_bitmap_page(
        plan,
        allocators,
        active_pages,
        format,
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
    )
}

fn reserve_bitmap_page(
    plan: &mut GlyphAtlasBitmapRunPlan,
    allocators: &mut BTreeMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>,
    active_pages: &mut BTreeMap<GlyphAtlasFormat, GlyphAtlasPageKey>,
    format: GlyphAtlasFormat,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
) -> Result<GlyphAtlasPageKey, GlyphAtlasBitmapAllocationFailureReason> {
    let reservation =
        plan.atlas
            .reserve_page_for_format(format, page_size, frame_index, max_pages_per_format);
    let Some(page) = reservation.page else {
        return Err(GlyphAtlasBitmapAllocationFailureReason::PageReservationBlocked);
    };

    let page_key = page.key;
    if matches!(
        reservation.decision,
        GlyphAtlasPageResidencyDecision::Evict(_)
    ) {
        plan.rebuilt_pages.push(page_key);
        mark_bitmap_dirty(
            &mut plan.dirty_pages,
            page_key,
            GlyphAtlasRect {
                x: 0,
                y: 0,
                width: page_size.x,
                height: page_size.y,
            },
        );
    }
    allocators.insert(
        page_key,
        GlyphAtlasShelfAllocator::new(page_key, page_size, padding_px),
    );
    active_pages.insert(format, page_key);
    Ok(page_key)
}

fn allocate_without_mutating_on_failure(
    allocators: &mut BTreeMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>,
    page_key: GlyphAtlasPageKey,
    size: UVec2,
) -> Option<GlyphAtlasAllocation> {
    let allocator = allocators.get(&page_key)?;
    let mut trial = allocator.clone();
    let allocation = trial.allocate(size)?;
    allocators.insert(page_key, trial);
    Some(allocation)
}

pub(super) fn mark_bitmap_dirty(
    dirty_pages: &mut Vec<GlyphAtlasDirtyPage>,
    page_key: GlyphAtlasPageKey,
    rect: GlyphAtlasRect,
) {
    if let Some(page) = dirty_pages
        .iter_mut()
        .find(|page| page.page_key() == page_key)
    {
        page.mark_dirty(page_key, rect);
    } else {
        let mut page = GlyphAtlasDirtyPage::new(page_key);
        page.mark_dirty(page_key, rect);
        dirty_pages.push(page);
    }
}
