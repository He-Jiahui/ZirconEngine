use std::collections::BTreeMap;

use crate::core::math::UVec2;

use super::super::{
    GlyphAtlasAllocation, GlyphAtlasDirtyPage, GlyphAtlasFormat, GlyphAtlasPageKey,
    GlyphAtlasPageResidencyDecision, GlyphAtlasRect, GlyphAtlasShelfAllocator,
};
use super::failure::GlyphAtlasBitmapAllocationFailureReason;
use super::types::{
    GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSlotInvalidation, GlyphAtlasBitmapSource,
};

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
    let reservation = plan.atlas.reserve_rebuildable_page_for_format(
        format,
        page_size,
        frame_index,
        max_pages_per_format,
    );
    let Some(page) = reservation.page else {
        return Err(GlyphAtlasBitmapAllocationFailureReason::PageReservationBlocked);
    };

    let page_key = page.key;
    if matches!(
        reservation.decision,
        GlyphAtlasPageResidencyDecision::Evict(_)
    ) {
        record_bitmap_page_rebuild(plan, page_key, page.generation, page_size);
    }
    allocators.insert(
        page_key,
        GlyphAtlasShelfAllocator::new(page_key, page_size, padding_px),
    );
    active_pages.insert(format, page_key);
    Ok(page_key)
}

pub(super) fn record_bitmap_page_rebuild(
    plan: &mut GlyphAtlasBitmapRunPlan,
    page_key: GlyphAtlasPageKey,
    page_generation: u64,
    page_size: UVec2,
) {
    plan.rebuilt_pages.push(page_key);
    plan.slot_invalidations
        .push(GlyphAtlasBitmapSlotInvalidation {
            page_key,
            page_generation,
        });
    mark_full_bitmap_page_dirty(
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

fn mark_full_bitmap_page_dirty(
    dirty_pages: &mut Vec<GlyphAtlasDirtyPage>,
    page_key: GlyphAtlasPageKey,
    page_rect: GlyphAtlasRect,
) {
    if let Some(page) = dirty_pages
        .iter_mut()
        .find(|page| page.page_key() == page_key)
    {
        page.mark_full_page_dirty(page_key, page_rect);
    } else {
        let mut page = GlyphAtlasDirtyPage::new(page_key);
        page.mark_full_page_dirty(page_key, page_rect);
        dirty_pages.push(page);
    }
}

fn allocate_without_mutating_on_failure(
    allocators: &mut BTreeMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>,
    page_key: GlyphAtlasPageKey,
    size: UVec2,
) -> Option<GlyphAtlasAllocation> {
    allocators.get_mut(&page_key)?.allocate(size)
}

pub(super) fn mark_bitmap_dirty(
    dirty_pages: &mut Vec<GlyphAtlasDirtyPage>,
    page_key: GlyphAtlasPageKey,
    rect: GlyphAtlasRect,
    retained_regions: Option<&[GlyphAtlasRect]>,
    has_replayable_shadow: bool,
) {
    if let Some(page) = dirty_pages
        .iter_mut()
        .find(|page| page.page_key() == page_key)
    {
        page.mark_dirty(page_key, rect);
    } else {
        let mut page = match (retained_regions, has_replayable_shadow) {
            (Some(retained_regions), true) => {
                GlyphAtlasDirtyPage::new_with_replayable_shadow(page_key, retained_regions.to_vec())
            }
            (Some(retained_regions), false) => {
                GlyphAtlasDirtyPage::new_with_retained_regions(page_key, retained_regions.to_vec())
            }
            (None, _) => GlyphAtlasDirtyPage::new(page_key),
        };
        page.mark_dirty(page_key, rect);
        dirty_pages.push(page);
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::math::UVec2;

    use super::{
        GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasShelfAllocator,
        allocate_without_mutating_on_failure,
    };

    #[test]
    fn optimization_batch_di_direct_bitmap_allocation_matches_trial_commit() {
        let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 3);
        let mut legacy = allocator_map(page_key, UVec2::new(64, 32));
        let mut optimized = legacy.clone();

        for size in [
            UVec2::new(20, 8),
            UVec2::new(50, 30),
            UVec2::new(20, 8),
            UVec2::new(20, 8),
        ] {
            assert_eq!(
                legacy_trial_commit(&mut legacy, page_key, size),
                allocate_without_mutating_on_failure(&mut optimized, page_key, size)
            );
            assert_eq!(optimized, legacy);
        }
    }

    #[test]
    fn optimization_batch_di_bitmap_allocation_uses_direct_mutation_source() {
        let source = include_str!("allocation.rs");
        let helper = source
            .split("fn allocate_without_mutating_on_failure")
            .nth(1)
            .expect("allocation helper")
            .split("pub(super) fn mark_bitmap_dirty")
            .next()
            .expect("helper body");

        assert!(helper.contains("allocators.get_mut(&page_key)?.allocate(size)"));
        assert!(!helper.contains("allocator.clone()"));
        assert!(!helper.contains("allocators.insert(page_key, trial)"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_di_direct_bitmap_allocation_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const ALLOCATIONS_PER_SAMPLE: usize = 65_536;

        let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 7);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_allocations(page_key, ALLOCATIONS_PER_SAMPLE, true));
                optimized_samples.push(measure_allocations(
                    page_key,
                    ALLOCATIONS_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples.push(measure_allocations(
                    page_key,
                    ALLOCATIONS_PER_SAMPLE,
                    false,
                ));
                legacy_samples.push(measure_allocations(page_key, ALLOCATIONS_PER_SAMPLE, true));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME417_DIRECT_BITMAP_SHELF_ALLOCATION_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "direct bitmap shelf allocation p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn allocator_map(
        page_key: GlyphAtlasPageKey,
        page_size: UVec2,
    ) -> BTreeMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator> {
        BTreeMap::from([(
            page_key,
            GlyphAtlasShelfAllocator::new(page_key, page_size, 0),
        )])
    }

    fn legacy_trial_commit(
        allocators: &mut BTreeMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>,
        page_key: GlyphAtlasPageKey,
        size: UVec2,
    ) -> Option<super::GlyphAtlasAllocation> {
        let allocator = allocators.get(&page_key)?;
        let mut trial = allocator.clone();
        let allocation = trial.allocate(size)?;
        allocators.insert(page_key, trial);
        Some(allocation)
    }

    fn measure_allocations(page_key: GlyphAtlasPageKey, allocations: usize, legacy: bool) -> u128 {
        let mut allocators = allocator_map(page_key, UVec2::new(allocations as u32, 1));
        let size = UVec2::ONE;
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..allocations {
            let allocation = if legacy {
                legacy_trial_commit(black_box(&mut allocators), page_key, size)
            } else {
                allocate_without_mutating_on_failure(black_box(&mut allocators), page_key, size)
            }
            .expect("benchmark allocation fits page");
            checksum = checksum.wrapping_add(u64::from(allocation.rect.x));
        }
        black_box((checksum, allocators));
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[index]
    }
}
