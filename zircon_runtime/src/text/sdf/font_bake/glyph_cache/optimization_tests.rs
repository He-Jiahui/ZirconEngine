use std::collections::HashSet;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::*;
use crate::text::FontWeight;
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasPageKey};
use crate::text::sdf::{SdfAtlasRect, SdfBakeParams, SdfGlyphMetrics};

use super::super::RawBakedGlyphSource;

const SAMPLE_PAIRS: usize = 21;

#[test]
fn runtime80_batch_glyph_cache_borrowed_protection_set_preserves_key_identity() {
    let slots = glyph_slots(8, 64);
    let protected = protected_glyph_keys(&slots);

    assert_eq!(protected.len(), slots.len());
    for slot in &slots {
        let protected_key = protected
            .get(&&slot.key)
            .expect("every visible slot must remain protected");
        assert!(std::ptr::eq(*protected_key, &slot.key));
    }
}

#[test]
fn runtime80_batch_glyph_cache_insert_preserves_replacement_byte_accounting() {
    let mut cache = SdfFontBakeCache::new();
    let key = glyph_key('A', 64);

    cache.insert_baked_glyph(key.clone(), raw_glyph(3));
    cache.insert_baked_glyph(key, raw_glyph(5));

    assert_eq!(cache.glyphs.len(), 1);
    assert_eq!(cache.resident_baked_glyph_byte_count, 5);
}

#[test]
#[ignore = "release performance evidence"]
fn runtime80_batch_glyph_cache_borrowed_protection_set_benchmark_evidence() {
    const SLOTS: usize = 16_384;
    const KEY_TEXT_BYTES: usize = 256;
    const ARC_FIELDS_PER_KEY: usize = 3;

    let slots = glyph_slots(SLOTS, KEY_TEXT_BYTES);
    let mut legacy = || legacy_owned_protected_glyph_keys(black_box(&slots)).len();
    let mut optimized = || protected_glyph_keys(black_box(&slots)).len();

    assert_eq!(black_box(legacy()), SLOTS);
    assert_eq!(black_box(optimized()), SLOTS);
    let (legacy_ns, optimized_ns) = paired_samples(&mut legacy, &mut optimized, SLOTS);
    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "borrowed protected glyph set P95 must be at least 25% below owned key cloning: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME80_BORROWED_PROTECTED_GLYPH_SET_BENCH_V1 slots={SLOTS} key_text_bytes={KEY_TEXT_BYTES} arc_fields_per_key={ARC_FIELDS_PER_KEY} sample_pairs={SAMPLE_PAIRS} legacy_key_clones={SLOTS} optimized_key_clones=0 legacy_arc_refcount_increments={} optimized_arc_refcount_increments=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        SLOTS * ARC_FIELDS_PER_KEY,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

#[test]
#[ignore = "release performance evidence"]
fn runtime80_batch_glyph_cache_single_probe_byte_accounting_benchmark_evidence() {
    const INSERTS: usize = 256;
    const KEY_TEXT_BYTES: usize = 2_048;
    const BITMAP_BYTES: usize = 64;

    let key = glyph_key('A', KEY_TEXT_BYTES);
    let glyph = raw_glyph(BITMAP_BYTES);
    let mut legacy_cache = SdfFontBakeCache::new();
    let mut optimized_cache = SdfFontBakeCache::new();
    let mut legacy = || {
        for _ in 0..INSERTS {
            legacy_insert_baked_glyph(&mut legacy_cache, key.clone(), glyph.clone());
        }
        legacy_cache.resident_baked_glyph_byte_count
    };
    let mut optimized = || {
        for _ in 0..INSERTS {
            optimized_cache.insert_baked_glyph(key.clone(), glyph.clone());
        }
        optimized_cache.resident_baked_glyph_byte_count
    };

    assert_eq!(black_box(legacy()), BITMAP_BYTES);
    assert_eq!(black_box(optimized()), BITMAP_BYTES);
    let (legacy_ns, optimized_ns) = paired_samples(&mut legacy, &mut optimized, BITMAP_BYTES);
    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "single-probe glyph byte accounting P95 must be at least 10% below insert-plus-get accounting: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME80_SINGLE_PROBE_GLYPH_BYTE_ACCOUNTING_BENCH_V1 inserts={INSERTS} key_text_bytes={KEY_TEXT_BYTES} bitmap_bytes={BITMAP_BYTES} sample_pairs={SAMPLE_PAIRS} legacy_glyph_map_probes_per_insert=2 optimized_glyph_map_probes_per_insert=1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn glyph_slots(count: usize, key_text_bytes: usize) -> Vec<SdfAtlasSlot> {
    (0..count)
        .map(|index| SdfAtlasSlot {
            key: glyph_key(
                char::from_u32(0x1_000 + index as u32).expect("fixture Unicode scalar"),
                key_text_bytes,
            ),
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0),
            rect: SdfAtlasRect {
                x: index as u32,
                y: 0,
                width: 1,
                height: 1,
            },
        })
        .collect()
}

fn glyph_key(glyph: char, key_text_bytes: usize) -> SdfAtlasGlyphKey {
    let payload = Arc::<str>::from("x".repeat(key_text_bytes));
    SdfAtlasGlyphKey {
        glyph,
        glyph_id: None,
        font_id: None,
        font_instance_id: None,
        font: Some(Arc::clone(&payload)),
        font_family: Some(Arc::clone(&payload)),
        language: Some(payload),
        font_weight: FontWeight::NORMAL.0,
        bake_params: SdfBakeParams::default(),
    }
}

fn raw_glyph(bitmap_bytes: usize) -> RawBakedGlyph {
    RawBakedGlyph {
        metrics: SdfGlyphMetrics::default(),
        bitmap: Arc::from(vec![1_u8; bitmap_bytes]),
        visible: true,
        generation_error: None,
        source: RawBakedGlyphSource::Dynamic,
    }
}

fn legacy_owned_protected_glyph_keys(slots: &[SdfAtlasSlot]) -> HashSet<SdfAtlasGlyphKey> {
    slots.iter().map(|slot| slot.key.clone()).collect()
}

fn legacy_insert_baked_glyph(
    cache: &mut SdfFontBakeCache,
    key: SdfAtlasGlyphKey,
    glyph: RawBakedGlyph,
) {
    cache.measured_glyphs.insert(key.clone(), glyph.metrics);
    if let Some(previous) = cache.glyphs.insert(key.clone(), glyph) {
        cache.resident_baked_glyph_byte_count = cache
            .resident_baked_glyph_byte_count
            .saturating_sub(previous.bitmap.len());
    }
    cache.resident_baked_glyph_byte_count = cache
        .resident_baked_glyph_byte_count
        .saturating_add(cache.glyphs.get(&key).map_or(0, |glyph| glyph.bitmap.len()));
    cache.touch_cached_glyph_key(key);
}

fn paired_samples(
    legacy: &mut impl FnMut() -> usize,
    optimized: &mut impl FnMut() -> usize,
    expected: usize,
) -> (Vec<u128>, Vec<u128>) {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(legacy, expected));
            optimized_ns.push(measure_ns(optimized, expected));
        } else {
            optimized_ns.push(measure_ns(optimized, expected));
            legacy_ns.push(measure_ns(legacy, expected));
        }
    }
    (legacy_ns, optimized_ns)
}

fn measure_ns(operation: &mut impl FnMut() -> usize, expected: usize) -> u128 {
    let started = Instant::now();
    assert_eq!(black_box(operation()), expected);
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
