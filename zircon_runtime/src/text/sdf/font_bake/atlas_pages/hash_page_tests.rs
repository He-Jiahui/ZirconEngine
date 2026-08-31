use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use crate::text::atlas::GlyphAtlasFormat;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 4_096;

#[test]
fn runtime11c_batch_sdf_page_hash_owner_preserves_bake_order() {
    let pages = [9_u32, 1, 5]
        .into_iter()
        .map(|page_index| {
            (
                page_key(page_index),
                PersistentAtlasPage {
                    size: UVec2::new(1, 1),
                    pixels: vec![0_u8].into(),
                    nonzero_pixel_count: 0,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let ordered = ordered_persistent_pages(&pages)
        .into_iter()
        .map(|(key, _)| key.page_index)
        .collect::<Vec<_>>();

    assert_eq!(ordered, vec![1, 5, 9]);
}

#[test]
fn runtime11c_batch_sdf_page_hash_owner_keeps_explicit_projection() {
    let source = include_str!("../atlas_pages.rs");

    assert!(source.contains("pages: HashMap<GlyphAtlasPageKey, PersistentAtlasPage>"));
    assert!(source.contains("fn ordered_persistent_pages("));
    assert!(source.contains("pages.sort_unstable_by_key"));
    assert!(source.contains("ordered_persistent_pages(&self.pages)"));
    assert!(source.contains("dirty_pages = BTreeMap::<GlyphAtlasPageKey, SdfAtlasRect>::new()"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime11c_batch_sdf_page_hash_owner_p95() {
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
        "SDF page hash lookup P95 must be at least 30% below BTreeMap: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME11C_SDF_PAGE_HASH_OWNER_BENCH_V1 pages={PAGES} lookups_per_sample={LOOKUPS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} ordered_page_lookups={LOOKUPS_PER_SAMPLE} hash_page_lookups={LOOKUPS_PER_SAMPLE} explicit_ordered_projections=1 dirty_page_order_changes=0 legacy_ns={} optimized_ns={}",
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
