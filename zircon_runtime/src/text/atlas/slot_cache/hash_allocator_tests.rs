use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use crate::text::atlas::GlyphAtlasFormat;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 4_096;

#[test]
fn runtime11c_batch_glyph_allocator_hash_index_isolates_page_lifetimes() {
    let first_page = page_key(1);
    let second_page = page_key(2);
    let mut cache = GlyphAtlasSlotCache::default();
    let page_size = UVec2::new(64, 64);
    let content_size = UVec2::new(8, 8);

    cache
        .allocate(first_page, page_size, 0, content_size)
        .expect("first page allocation must succeed");
    let first_second_page_allocation = cache
        .allocate(second_page, page_size, 0, content_size)
        .expect("second page allocation must succeed");
    let next_second_page_allocation = cache
        .allocate(second_page, page_size, 0, content_size)
        .expect("second page must retain allocator progress");

    cache.invalidate_page(first_page);

    assert_eq!(cache.allocators.len(), 1);
    assert!(cache.allocators.contains_key(&second_page));
    assert_ne!(
        first_second_page_allocation.rect,
        next_second_page_allocation.rect
    );
    assert!(
        cache
            .allocate(second_page, page_size, 0, content_size)
            .is_some()
    );
}

#[test]
fn runtime11c_batch_glyph_allocator_hash_index_keeps_ordered_diagnostics() {
    let source = include_str!("../slot_cache.rs");

    assert!(source.contains("allocators: HashMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>"));
    assert!(
        source.contains(
            "slot_rects_by_page(&self) -> BTreeMap<GlyphAtlasPageKey, Vec<GlyphAtlasRect>>"
        )
    );
    assert!(source.contains("let mut rects_by_page = BTreeMap::new();"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime11c_batch_glyph_allocator_hash_index_p95() {
    const PAGES: usize = 16_384;
    let keys = (0..PAGES)
        .map(|page_index| page_key(page_index as u32))
        .collect::<Vec<_>>();
    let legacy = keys
        .iter()
        .copied()
        .enumerate()
        .map(|(index, key)| (key, index))
        .collect::<BTreeMap<_, _>>();
    let optimized = keys
        .iter()
        .copied()
        .enumerate()
        .map(|(index, key)| (key, index))
        .collect::<HashMap<_, _>>();
    let target = keys.last().expect("benchmark page set must not be empty");

    let mut legacy_lookup = || repeated_lookup(&legacy, target);
    let mut optimized_lookup = || repeated_lookup(&optimized, target);
    assert_eq!(black_box(legacy_lookup()), black_box(optimized_lookup()));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(&mut legacy_lookup));
            optimized_ns.push(measure_ns(&mut optimized_lookup));
        } else {
            optimized_ns.push(measure_ns(&mut optimized_lookup));
            legacy_ns.push(measure_ns(&mut legacy_lookup));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "glyph allocator hash lookup P95 must be at least 30% below BTreeMap: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME11C_GLYPH_ALLOCATOR_HASH_INDEX_BENCH_V1 pages={PAGES} lookups_per_sample={LOOKUPS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} ordered_allocator_lookups={LOOKUPS_PER_SAMPLE} hash_allocator_lookups={LOOKUPS_PER_SAMPLE} ordered_diagnostic_changes=0 allocator_clone_changes=0 legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn page_key(page_index: u32) -> GlyphAtlasPageKey {
    GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, page_index)
}

fn repeated_lookup<M>(map: &M, key: &GlyphAtlasPageKey) -> usize
where
    M: LookupMap,
{
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum = checksum.wrapping_add(black_box(map.lookup(black_box(key))));
    }
    black_box(checksum)
}

trait LookupMap {
    fn lookup(&self, key: &GlyphAtlasPageKey) -> usize;
}

impl LookupMap for BTreeMap<GlyphAtlasPageKey, usize> {
    fn lookup(&self, key: &GlyphAtlasPageKey) -> usize {
        *self.get(key).expect("legacy benchmark page must exist")
    }
}

impl LookupMap for HashMap<GlyphAtlasPageKey, usize> {
    fn lookup(&self, key: &GlyphAtlasPageKey) -> usize {
        *self.get(key).expect("optimized benchmark page must exist")
    }
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
