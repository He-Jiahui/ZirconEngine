use std::{hint::black_box, time::Instant};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826az_glyph_page_index_preserves_smallest_sparse_gap() {
    let pages = vec![
        resident_page(GlyphAtlasFormat::AlphaMask, 0),
        resident_page(GlyphAtlasFormat::Sdf, 1),
        resident_page(GlyphAtlasFormat::AlphaMask, 2),
        resident_page(GlyphAtlasFormat::AlphaMask, 129),
        resident_page(GlyphAtlasFormat::AlphaMask, 130),
    ];

    assert_eq!(
        page_residency_decision(&pages, GlyphAtlasFormat::AlphaMask, 5),
        GlyphAtlasPageResidencyDecision::Allocate(GlyphAtlasPageKey::new(
            GlyphAtlasFormat::AlphaMask,
            1,
        ))
    );
}

#[test]
fn optimization_batch_20260826az_glyph_page_index_uses_single_occupancy_summary() {
    let source = include_str!("../page_residency.rs");
    let decision = bounded_source(
        source,
        "pub(crate) fn page_residency_decision(",
        "pub(crate) fn page_rebuild_residency_decision(",
    );
    let index = bounded_source(source, "fn page_format_occupancy(", "#[cfg(test)]");

    assert!(source.contains("struct PageFormatOccupancy"));
    assert!(source.contains("INLINE_PAGE_INDEX_CAPACITY"));
    assert!(decision.contains("page_format_occupancy"));
    assert!(index.contains("low_indices.trailing_ones()"));
    assert!(index.contains("vec![false; occupancy.page_count + 1]"));
    assert!(!index.contains("while pages"));
    assert!(!index.contains("pages.iter().any"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826az_glyph_page_single_pass_index_p95() {
    const PAGE_COUNT: usize = 2_048;
    const DECISIONS: usize = 2;
    let pages = (0..PAGE_COUNT)
        .map(|index| resident_page(GlyphAtlasFormat::AlphaMask, index as u32))
        .collect::<Vec<_>>();
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(DECISIONS, || legacy_allocate(&pages)));
            optimized_ns.push(measure_ns(DECISIONS, || optimized_allocate(&pages)));
        } else {
            optimized_ns.push(measure_ns(DECISIONS, || optimized_allocate(&pages)));
            legacy_ns.push(measure_ns(DECISIONS, || legacy_allocate(&pages)));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns,
        "single-pass glyph page index P95 must be at least 90% below repeated page scans: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME11C_GLYPH_PAGE_SINGLE_PASS_INDEX_BENCH_V1 pages={PAGE_COUNT} decisions_per_sample={DECISIONS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_page_visits_per_sample={} optimized_page_visits_per_sample={} optimized_bitmap_probes_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        PAGE_COUNT * (PAGE_COUNT + 5) / 2 * DECISIONS,
        PAGE_COUNT * 2 * DECISIONS,
        (PAGE_COUNT + 1) * DECISIONS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn optimized_allocate(pages: &[GlyphAtlasResidentPage]) -> usize {
    let expected = GlyphAtlasPageResidencyDecision::Allocate(GlyphAtlasPageKey::new(
        GlyphAtlasFormat::AlphaMask,
        pages.len() as u32,
    ));
    usize::from(
        page_residency_decision(
            black_box(pages),
            GlyphAtlasFormat::AlphaMask,
            pages.len() + 1,
        ) == expected,
    )
}

fn legacy_allocate(pages: &[GlyphAtlasResidentPage]) -> usize {
    let format = GlyphAtlasFormat::AlphaMask;
    let count = pages
        .iter()
        .filter(|page| page.key().format == format)
        .count();
    let mut page_index = 0;
    while pages
        .iter()
        .any(|page| page.key() == GlyphAtlasPageKey::new(format, page_index))
    {
        page_index = page_index.saturating_add(1);
    }
    usize::from(count == pages.len() && page_index == pages.len() as u32)
}

fn resident_page(format: GlyphAtlasFormat, page_index: u32) -> GlyphAtlasResidentPage {
    GlyphAtlasResidentPage::from_existing_page(GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(format, page_index),
        UVec2::new(1024, 1024),
    ))
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
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
