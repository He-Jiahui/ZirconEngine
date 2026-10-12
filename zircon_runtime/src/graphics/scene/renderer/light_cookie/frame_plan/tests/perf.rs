use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const COOKIE_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn runtime95_frame_metadata_cookie_index_matches_legacy_sort_and_dedup() {
    let texture = ResourceId::from_stable_label("runtime://cookie/shared");
    let replacement = ResourceId::from_stable_label("runtime://cookie/replacement");
    let mut cookies = (0..96)
        .rev()
        .map(|light_id| LightCookieData {
            light_id,
            texture,
            projection: CookieProjection::Spot,
        })
        .collect::<Vec<_>>();
    cookies.push(LightCookieData {
        light_id: 3,
        texture: replacement,
        projection: CookieProjection::PointOctahedral,
    });

    let optimized = build_cookie_frame_plan(&cookies);
    let legacy = legacy_build_cookie_frame_plan(&cookies);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.entries().len(), COOKIE_ATLAS_MAX_ENTRIES);
    assert_eq!(optimized.entries()[3].texture, replacement);
}

#[test]
#[ignore = "release-only cookie candidate benchmark"]
fn runtime95_frame_metadata_cookie_release_benchmark_evidence() {
    let texture = ResourceId::from_stable_label("runtime://cookie/benchmark");
    let cookies = (0..COOKIE_COUNT as u64)
        .rev()
        .map(|light_id| LightCookieData {
            light_id,
            texture,
            projection: CookieProjection::Spot,
        })
        .collect::<Vec<_>>();

    black_box(time_legacy(&cookies));
    black_box(time_optimized(&cookies));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(time_legacy(&cookies));
            optimized_samples.push(time_optimized(&cookies));
        } else {
            optimized_samples.push(time_optimized(&cookies));
            legacy_samples.push(time_legacy(&cookies));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    println!(
        "RUNTIME95_COOKIE_CANDIDATE_PERF cookies=65536 atlas_entries=64 pairs=21 order=alternating percentile=nearest-rank legacy_tree_nodes=65536 optimized_tree_nodes=0 optimized_index_entries=65536 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        nearest_rank(&legacy_samples, 50),
        legacy_p95_ns,
        nearest_rank(&optimized_samples, 50),
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "sorted cookie index must reduce P95 by at least 25%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_build_cookie_frame_plan(cookies: &[LightCookieData]) -> CookieFramePlan {
    let unique = cookies
        .iter()
        .map(|cookie| (cookie.light_id, cookie))
        .collect::<BTreeMap<_, _>>();
    let cell = 1.0 / COOKIE_ATLAS_GRID_SIZE as f32;
    let entries = unique
        .into_iter()
        .take(COOKIE_ATLAS_MAX_ENTRIES)
        .enumerate()
        .map(|(slot, (light_id, cookie))| {
            let slot = slot as u32;
            let x = slot % COOKIE_ATLAS_GRID_SIZE;
            let y = slot / COOKIE_ATLAS_GRID_SIZE;
            let (projection, wrap, offset, scale) = projection_metadata(cookie.projection);
            CookieAtlasEntry {
                slot,
                light_id,
                texture: cookie.texture,
                metadata: CookieGpuMetadata {
                    uv_rect: [x as f32 * cell, y as f32 * cell, cell, cell],
                    misc: [projection, wrap, 0, 0],
                    directional_offset_scale: [offset.x, offset.y, scale.x, scale.y],
                },
            }
        })
        .collect();
    CookieFramePlan { entries }
}

fn time_legacy(cookies: &[LightCookieData]) -> u128 {
    let started = Instant::now();
    let plan = legacy_build_cookie_frame_plan(black_box(cookies));
    let elapsed = started.elapsed().as_nanos();
    black_box(plan);
    elapsed
}

fn time_optimized(cookies: &[LightCookieData]) -> u128 {
    let started = Instant::now();
    let plan = build_cookie_frame_plan(black_box(cookies));
    let elapsed = started.elapsed().as_nanos();
    black_box(plan);
    elapsed
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
