use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use crate::core::math::UVec2;

use super::{
    allocate_without_mutating_on_failure, GlyphAtlasFormat, GlyphAtlasPageKey,
    GlyphAtlasShelfAllocator,
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
    let source = include_str!("../allocation.rs");
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
            optimized_samples.push(measure_allocations(page_key, ALLOCATIONS_PER_SAMPLE, false));
        } else {
            optimized_samples.push(measure_allocations(page_key, ALLOCATIONS_PER_SAMPLE, false));
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
