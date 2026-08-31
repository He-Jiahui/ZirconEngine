use std::hint::black_box;
use std::time::Instant;

use super::{GlyphAtlasDirtyPage, GlyphAtlasPageKey, GlyphAtlasRect};
use crate::text::atlas::GlyphAtlasFormat;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 4;
const REGIONS_PER_BUILD: usize = 160;

#[test]
fn optimization_batch_20260829as_runtime319_single_frontier_matches_legacy_regions() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let regions = [
        atlas_rect(0, 0, 4, 4),
        atlas_rect(10, 0, 3, 4),
        atlas_rect(4, 0, 6, 4),
        atlas_rect(40, 10, 2, 8),
        atlas_rect(40, 18, 2, 4),
        atlas_rect(80, 2, 5, 5),
    ];
    let mut legacy = GlyphAtlasDirtyPage::new(page_key);
    let mut optimized = GlyphAtlasDirtyPage::new(page_key);

    for region in regions {
        legacy_mark_dirty(&mut legacy, page_key, region);
        optimized.mark_dirty(page_key, region);
    }

    assert_eq!(optimized.regions, legacy.regions);
    assert_eq!(optimized.merged_rect, legacy.merged_rect);
}

#[test]
fn optimization_batch_20260829as_runtime319_new_region_frontier_closes_merge_chain() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);

    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 4, 4));
    dirty_page.mark_dirty(page_key, atlas_rect(8, 0, 4, 4));
    dirty_page.mark_dirty(page_key, atlas_rect(4, 0, 4, 4));

    assert_eq!(dirty_page.regions(), &[atlas_rect(0, 0, 12, 4)]);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829as_runtime319_single_frontier_glyph_dirty_merge_bench() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(page_key, false));
            optimized_samples.push(measure(page_key, true));
        } else {
            optimized_samples.push(measure(page_key, true));
            legacy_samples.push(measure(page_key, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    let legacy_pair_checks_per_build = (REGIONS_PER_BUILD + 1)
        .saturating_mul(REGIONS_PER_BUILD)
        .saturating_mul(REGIONS_PER_BUILD - 1)
        / 6;
    let optimized_pair_checks_per_build =
        REGIONS_PER_BUILD.saturating_mul(REGIONS_PER_BUILD - 1) / 2;
    println!(
        "RUNTIME319_SINGLE_FRONTIER_GLYPH_DIRTY_MERGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} regions_per_build={REGIONS_PER_BUILD} \
legacy_pair_checks_per_build={legacy_pair_checks_per_build} \
optimized_pair_checks_per_build={optimized_pair_checks_per_build} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_mark_dirty(
    dirty_page: &mut GlyphAtlasDirtyPage,
    page_key: GlyphAtlasPageKey,
    rect: GlyphAtlasRect,
) {
    if page_key != dirty_page.page_key || rect.width == 0 || rect.height == 0 {
        return;
    }
    dirty_page.merged_rect = Some(match dirty_page.merged_rect {
        Some(existing) => existing.union(rect),
        None => rect,
    });
    if dirty_page.full_page_requested {
        return;
    }

    dirty_page.regions.push(rect);
    loop {
        let Some((left_index, right_index, merged)) = legacy_safe_region_merge(dirty_page) else {
            break;
        };
        dirty_page.regions[left_index] = merged;
        dirty_page.regions.remove(right_index);
    }
    dirty_page.enforce_write_limit_with_shadow();
}

fn legacy_safe_region_merge(
    dirty_page: &GlyphAtlasDirtyPage,
) -> Option<(usize, usize, GlyphAtlasRect)> {
    let mut best = None;
    for left_index in 0..dirty_page.regions.len() {
        for right_index in left_index.saturating_add(1)..dirty_page.regions.len() {
            let left = dirty_page.regions[left_index];
            let right = dirty_page.regions[right_index];
            let merged = left.union(right);
            if dirty_page.intersects_retained_region(merged)
                && !dirty_page.can_replay_retained_pixels
            {
                continue;
            }
            let extra_byte_cost = dirty_page.merge_extra_byte_cost(left, right, merged);
            if dirty_page.has_exact_coverage(left, right, merged)
                || ((dirty_page.can_clear_unretained_pixels
                    || dirty_page.can_replay_retained_pixels)
                    && extra_byte_cost <= super::GLYPH_ATLAS_DIRTY_MAX_MERGE_EXTRA_BYTES)
            {
                if best.is_none_or(|(_, _, _, best_cost)| extra_byte_cost < best_cost) {
                    best = Some((left_index, right_index, merged, extra_byte_cost));
                }
            }
        }
    }
    best.map(|(left_index, right_index, merged, _)| (left_index, right_index, merged))
}

fn measure(page_key: GlyphAtlasPageKey, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);
        for index in 0..REGIONS_PER_BUILD {
            let rect = atlas_rect((index as u32).saturating_mul(3), 0, 1, 1);
            if optimized {
                dirty_page.mark_dirty(page_key, black_box(rect));
            } else {
                legacy_mark_dirty(&mut dirty_page, page_key, black_box(rect));
            }
        }
        checksum = checksum.wrapping_add(dirty_page.regions.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
