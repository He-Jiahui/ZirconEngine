use std::hint::black_box;
use std::time::Instant;

use super::*;

const PAGE_COUNT: u32 = 2_048;
const SAMPLE_PAIRS: usize = 21;

fn page_key(page_index: u32) -> GlyphAtlasPageKey {
    GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, page_index)
}

fn rect(x: u32, width: u32) -> SdfAtlasRect {
    SdfAtlasRect {
        x,
        y: 0,
        width,
        height: 4,
    }
}

#[test]
fn optimization_batch_20260826co_runtime132_binary_merge_preserves_union_and_page_order() {
    let mut cache = SdfAtlasCacheReport {
        dirty_pages: vec![
            SdfAtlasDirtyPageReport {
                page_key: page_key(2),
                dirty_rect: rect(2, 2),
            },
            SdfAtlasDirtyPageReport {
                page_key: page_key(0),
                dirty_rect: rect(4, 4),
            },
        ],
        ..SdfAtlasCacheReport::default()
    };
    let bake = [
        SdfAtlasBakeDirtyPage {
            page_key: page_key(1),
            dirty_rect: rect(8, 2),
        },
        SdfAtlasBakeDirtyPage {
            page_key: page_key(0),
            dirty_rect: rect(1, 4),
        },
    ];

    merge_sdf_bake_dirty_pages(&mut cache, &bake);

    assert_eq!(
        cache
            .dirty_pages
            .iter()
            .map(|page| page.page_key.page_index)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(cache.dirty_pages[0].dirty_rect, rect(1, 7));
    assert_eq!(cache.dirty_rect, Some(rect(1, 7)));
}

#[test]
fn optimization_batch_20260826co_runtime132_dirty_page_merge_uses_binary_index() {
    let source = include_str!("../sdf_upload.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let compact = source.split_whitespace().collect::<String>();

    assert!(compact.contains("dirty_pages.binary_search_by_key(&bake_page.page_key"));
    assert!(!compact.contains("dirty_pages.iter_mut().find"));
    assert!(!source.contains("dirty_pages.sort_by_key"));
}

fn legacy_merge(
    dirty_pages: &mut Vec<SdfAtlasDirtyPageReport>,
    bake_dirty_pages: &[SdfAtlasBakeDirtyPage],
) {
    for bake_page in bake_dirty_pages {
        if let Some(cache_page) = dirty_pages
            .iter_mut()
            .find(|cache_page| cache_page.page_key == bake_page.page_key)
        {
            cache_page.dirty_rect = union_rect(cache_page.dirty_rect, bake_page.dirty_rect);
        } else {
            dirty_pages.push(SdfAtlasDirtyPageReport {
                page_key: bake_page.page_key,
                dirty_rect: bake_page.dirty_rect,
            });
        }
    }
    dirty_pages.sort_by_key(|page| page.page_key);
}

fn optimized_merge(
    dirty_pages: &mut Vec<SdfAtlasDirtyPageReport>,
    bake_dirty_pages: &[SdfAtlasBakeDirtyPage],
) {
    for bake_page in bake_dirty_pages {
        match dirty_pages.binary_search_by_key(&bake_page.page_key, |page| page.page_key) {
            Ok(index) => {
                let page = &mut dirty_pages[index];
                page.dirty_rect = union_rect(page.dirty_rect, bake_page.dirty_rect);
            }
            Err(index) => dirty_pages.insert(
                index,
                SdfAtlasDirtyPageReport {
                    page_key: bake_page.page_key,
                    dirty_rect: bake_page.dirty_rect,
                },
            ),
        }
    }
}

fn benchmark_pages() -> (Vec<SdfAtlasDirtyPageReport>, Vec<SdfAtlasBakeDirtyPage>) {
    let dirty_pages = (0..PAGE_COUNT)
        .map(|page_index| SdfAtlasDirtyPageReport {
            page_key: page_key(page_index),
            dirty_rect: rect(page_index, 2),
        })
        .collect();
    let bake_pages = (0..PAGE_COUNT)
        .rev()
        .map(|page_index| SdfAtlasBakeDirtyPage {
            page_key: page_key(page_index),
            dirty_rect: rect(page_index + 1, 2),
        })
        .collect();
    (dirty_pages, bake_pages)
}

fn elapsed_ns(
    base: &[SdfAtlasDirtyPageReport],
    bake: &[SdfAtlasBakeDirtyPage],
    merge: fn(&mut Vec<SdfAtlasDirtyPageReport>, &[SdfAtlasBakeDirtyPage]),
) -> u128 {
    let mut pages = base.to_vec();
    let started = Instant::now();
    merge(&mut pages, bake);
    black_box(pages);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826co_runtime132_dirty_page_binary_merge_performance_evidence() {
    let (base, bake) = benchmark_pages();
    for _ in 0..3 {
        let mut legacy = base.clone();
        let mut optimized = base.clone();
        legacy_merge(&mut legacy, &bake);
        optimized_merge(&mut optimized, &bake);
        assert_eq!(black_box(legacy), optimized);
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_ns(&base, &bake, legacy_merge));
            optimized_samples.push(elapsed_ns(&base, &bake, optimized_merge));
        } else {
            optimized_samples.push(elapsed_ns(&base, &bake, optimized_merge));
            legacy_samples.push(elapsed_ns(&base, &bake, legacy_merge));
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples, 95);
    println!(
        "RUNTIME132_SDF_DIRTY_PAGE_BINARY_MERGE_BENCH_V1 sample_pairs={} page_count={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SAMPLE_PAIRS,
        PAGE_COUNT,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "binary dirty-page merge p95 must be at least 30% below linear merge and sort: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
