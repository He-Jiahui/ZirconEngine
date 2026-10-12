use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const OVERLAYS_PER_SAMPLE: usize = 131_072;

#[test]
fn optimization_batch_fj_editor396_refresh_overlay_preserves_bytes() {
    let diagnostics = populated_diagnostics();
    for fps in [0.0, 1.0, 59.94, 120.05, f32::INFINITY] {
        assert_eq!(
            refresh_overlay_text_with_fps(&diagnostics, fps),
            legacy_refresh_overlay_text(&diagnostics, fps)
        );
    }
    assert_eq!(
        refresh_overlay_text(&HostRefreshDiagnostics::default()),
        STARTUP_REFRESH_DIAGNOSTICS_OVERLAY
    );
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fj_editor396_direct_refresh_overlay_benchmark() {
    let diagnostics = populated_diagnostics();
    const FPS: f32 = 59.94;
    for _ in 0..4 {
        black_box(measure(legacy_refresh_overlay_text, &diagnostics, FPS));
        black_box(measure(refresh_overlay_text_with_fps, &diagnostics, FPS));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure(legacy_refresh_overlay_text, &diagnostics, FPS));
            optimized_samples.push(measure(refresh_overlay_text_with_fps, &diagnostics, FPS));
        } else {
            optimized_samples.push(measure(refresh_overlay_text_with_fps, &diagnostics, FPS));
            legacy_samples.push(measure(legacy_refresh_overlay_text, &diagnostics, FPS));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn populated_diagnostics() -> HostRefreshDiagnostics {
    HostRefreshDiagnostics {
        present_count: 1_234_567,
        full_paint_count: 234_567,
        region_paint_count: 1_000_000,
        painted_pixel_count: 9_876_543_210,
        slow_path_rebuild_count: 123,
        render_rebuild_count: 456,
        paint_only_request_count: 789,
        ..HostRefreshDiagnostics::default()
    }
}

fn legacy_refresh_overlay_text(diagnostics: &HostRefreshDiagnostics, fps: f32) -> String {
    format!(
        "FPS {fps:.1} | present {} | full {} | region {} | pixels {} | slow {} | render {} | paint-only {}",
        diagnostics.present_count,
        diagnostics.full_paint_count,
        diagnostics.region_paint_count,
        diagnostics.painted_pixel_count,
        diagnostics.slow_path_rebuild_count,
        diagnostics.render_rebuild_count,
        diagnostics.paint_only_request_count,
    )
}

fn measure(
    mut build: impl FnMut(&HostRefreshDiagnostics, f32) -> String,
    diagnostics: &HostRefreshDiagnostics,
    fps: f32,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..OVERLAYS_PER_SAMPLE {
        checksum = checksum.wrapping_add(black_box(build(black_box(diagnostics), fps)).len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR396_DIRECT_REFRESH_OVERLAY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} overlays_per_sample={OVERLAYS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
        "optimized p95 {optimized_p95}ns must be at most 75% of legacy p95 {legacy_p95}ns"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn optimization_batch_gw_editor578_overlay_with_invalidation_preserves_bytes() {
    let diagnostics = populated_diagnostics();
    let invalidation = HostInvalidationDiagnostics {
        slow_path_rebuild_count: 17,
        render_rebuild_count: 23,
        paint_only_request_count: 29,
    };
    let expected = diagnostics
        .clone()
        .with_invalidation_diagnostics(invalidation)
        .overlay_text();
    assert_eq!(
        refresh_overlay_text_with_invalidation(&diagnostics, invalidation),
        expected
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_gw_editor578_overlay_invalidation_single_pass_p95() {
    let diagnostics = populated_diagnostics();
    let invalidation = HostInvalidationDiagnostics {
        slow_path_rebuild_count: 17,
        render_rebuild_count: 23,
        paint_only_request_count: 29,
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_with_invalidation(&diagnostics, invalidation, false));
            optimized_samples.push(measure_with_invalidation(&diagnostics, invalidation, true));
        } else {
            optimized_samples.push(measure_with_invalidation(&diagnostics, invalidation, true));
            legacy_samples.push(measure_with_invalidation(&diagnostics, invalidation, false));
        }
    }
    let legacy_p95_ns = nearest_rank_p95(&legacy_samples);
    let optimized_p95_ns = nearest_rank_p95(&optimized_samples);
    println!(
        "EDITOR578_OVERLAY_INVALIDATION_SINGLE_PASS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} overlays_per_sample={OVERLAYS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
        "overlay invalidation path should avoid diagnostic clone: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_with_invalidation(
    diagnostics: &HostRefreshDiagnostics,
    invalidation: HostInvalidationDiagnostics,
    optimized: bool,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..OVERLAYS_PER_SAMPLE {
        let text = if optimized {
            refresh_overlay_text_with_invalidation(black_box(diagnostics), invalidation)
        } else {
            black_box(diagnostics)
                .clone()
                .with_invalidation_diagnostics(invalidation)
                .overlay_text()
        };
        checksum = checksum.wrapping_add(text.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}
