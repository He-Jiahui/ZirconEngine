use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::math::UVec2;

use super::super::super::{GlyphAtlasFormat, GlyphAtlasRect};
use super::*;

const PAGE_INDEX_COUNT: usize = 32_768;
const PAGE_LOOKUP_COUNT: usize = 262_144;
const SAMPLE_COUNT: usize = 17;

fn nearest_rank(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn page_keys() -> Vec<GlyphAtlasPageKey> {
    let formats = GlyphAtlasFormat::supported_formats();
    (0..PAGE_INDEX_COUNT)
        .map(|index| GlyphAtlasPageKey::new(formats[index % formats.len()], index as u32))
        .collect()
}

fn lookup_keys(page_keys: &[GlyphAtlasPageKey]) -> Vec<GlyphAtlasPageKey> {
    (0..PAGE_LOOKUP_COUNT)
        .map(|index| page_keys[(index * 32_749) % page_keys.len()])
        .collect()
}

fn ordered_lookup_sum(
    index: &BTreeMap<GlyphAtlasPageKey, usize>,
    lookups: &[GlyphAtlasPageKey],
) -> usize {
    lookups
        .iter()
        .filter_map(|page_key| index.get(page_key))
        .copied()
        .sum()
}

fn hash_lookup_sum(
    index: &HashMap<GlyphAtlasPageKey, usize>,
    lookups: &[GlyphAtlasPageKey],
) -> usize {
    lookups
        .iter()
        .filter_map(|page_key| index.get(page_key))
        .copied()
        .sum()
}

#[test]
fn runtime11c_batch_hash_shadow_store_preserves_generation_filter() {
    let page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 7),
        UVec2::new(8, 8),
    )
    .with_generation(3);
    let mut store = GlyphAtlasBitmapPageShadowStore::default();
    let mut commit = GlyphAtlasBitmapPageShadowCommit::default();
    commit.zero_initialized_pages.insert(page.key);
    store.apply(std::slice::from_ref(&page), commit);

    assert_eq!(store.bytes_for_page(&page), Some(&[0; 64][..]));

    let next_generation = page.clone().with_generation(4);
    store.apply(
        std::slice::from_ref(&next_generation),
        GlyphAtlasBitmapPageShadowCommit::default(),
    );
    assert!(store.bytes_for_page(&page).is_none());
    assert!(store.bytes_for_page(&next_generation).is_none());
}

#[test]
fn bitmap_page_shadow_report_exposes_residency_budget_and_rejections() {
    let page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 9),
        UVec2::new(8, 8),
    );
    let mut store = GlyphAtlasBitmapPageShadowStore::with_max_bytes(63);
    let mut commit = GlyphAtlasBitmapPageShadowCommit::default();
    commit.zero_initialized_pages.insert(page.key);
    commit.patches.push(GlyphAtlasBitmapPageShadowPatch {
        page_key: page.key,
        page_generation: page.generation,
        target_rect: GlyphAtlasRect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        },
        bytes_per_row: 1,
        bytes: vec![255].into(),
    });

    store.apply(std::slice::from_ref(&page), commit);

    assert_eq!(
        store.report(),
        GlyphAtlasBitmapPageShadowReport {
            resident_page_count: 0,
            resident_byte_count: 0,
            max_byte_count: 63,
            budget_rejection_count: 1,
        }
    );
}

#[test]
fn runtime11c_batch_page_shadow_uses_hash_indexes() {
    let source = include_str!("../store.rs");
    let production = source.split("mod optimization_tests").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeSet, HashMap};"));
    assert!(production.contains("pages: HashMap<GlyphAtlasPageKey"));
    assert_eq!(production.matches("collect::<HashMap<_, _>>()").count(), 2);
    assert_eq!(production.matches("collect::<BTreeSet<_>>()").count(), 1);
    assert!(!production.contains("BTreeMap"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime11c_batch_page_shadow_hash_index_performance_evidence() {
    let page_keys = page_keys();
    let lookups = lookup_keys(&page_keys);
    let ordered_index = page_keys
        .iter()
        .enumerate()
        .map(|(value, page_key)| (*page_key, value))
        .collect::<BTreeMap<_, _>>();
    let hash_index = page_keys
        .iter()
        .enumerate()
        .map(|(value, page_key)| (*page_key, value))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        ordered_lookup_sum(&ordered_index, &lookups),
        hash_lookup_sum(&hash_index, &lookups)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_lookup_sum(
                black_box(&ordered_index),
                black_box(&lookups),
            ));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_lookup_sum(black_box(&hash_index), black_box(&lookups)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_lookup_sum(black_box(&hash_index), black_box(&lookups)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_lookup_sum(
                black_box(&ordered_index),
                black_box(&lookups),
            ));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p50 = nearest_rank(&mut ordered_samples.clone(), 50);
    let ordered_p95 = nearest_rank(&mut ordered_samples, 95);
    let hash_p50 = nearest_rank(&mut hash_samples.clone(), 50);
    let hash_p95 = nearest_rank(&mut hash_samples, 95);
    println!(
        "RUNTIME11C_PAGE_SHADOW_HASH_INDEX_BENCH_V1 pages={PAGE_INDEX_COUNT} \
             lookups={PAGE_LOOKUP_COUNT} sample_pairs={SAMPLE_COUNT} \
             pair_order=alternating_ordered_even ordered_first_pairs=9 hash_first_pairs=8 \
             ordered_lookup_class=log_n hash_lookup_class=average_constant \
             ordered_p50_ns={} ordered_p95_ns={} hash_p50_ns={} hash_p95_ns={} \
             persistent_hash_indexes=3 ordered_zero_init_sets=1",
        ordered_p50.as_nanos(),
        ordered_p95.as_nanos(),
        hash_p50.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-index P95 {:?} exceeded 60% of ordered-index P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
