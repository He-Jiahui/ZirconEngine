use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::ui::retained_host::primitives::SharedString;

const SAMPLE_PAIRS: usize = 21;
const ITERATIONS_PER_SAMPLE: usize = 2_048;
const INFO_BYTES: usize = 16_384;

#[test]
fn optimization_batch_hp_editor597_owned_info_preserves_shared_string_content() {
    let info = "pane-info".repeat(256);

    let legacy = legacy_info_projection(&info);
    let optimized = optimized_info_projection(info);

    assert_eq!(optimized, legacy);
}

#[test]
fn optimization_batch_hp_editor597_pane_projection_moves_info_after_native_body() {
    let source = include_str!("../../pane_projection.rs");
    let projection = source
        .split("pub(super) fn pane_from_tab_with_template_v2_data")
        .nth(1)
        .expect("pane projection")
        .split("fn build_native_body")
        .next()
        .expect("pane projection body");
    let native_body = projection
        .find("let native_body = build_native_body(")
        .expect("native body precomputation");
    let pane = projection.find("PaneData {").expect("pane construction");

    assert!(native_body < pane);
    assert!(projection.contains("info: info.into(),"));
    assert!(projection.contains("native_body,"));
    assert!(!projection.contains("info: info.clone().into(),"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hp_editor597_pane_info_move_p95() {
    const MARKER: &str = "EDITOR597_PANE_INFO_MOVE_BENCH_V1";
    let info = "pane-information".repeat(INFO_BYTES / "pane-information".len());
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&info, false));
            optimized.push(measure(&info, true));
        } else {
            optimized.push(measure(&info, true));
            legacy.push(measure(&info, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4}"
    );
    assert!(
        ratio <= 0.35,
        "{MARKER} expected info move ratio <= 0.35, got {ratio:.4}"
    );
}

fn measure(info: &str, optimized: bool) -> Duration {
    let mut elapsed = Duration::ZERO;
    for _ in 0..ITERATIONS_PER_SAMPLE {
        let input = info.to_string();
        let start = Instant::now();
        let output = if optimized {
            optimized_info_projection(input)
        } else {
            legacy_info_projection(&input)
        };
        black_box(&output);
        elapsed += start.elapsed();
    }
    elapsed
}

fn legacy_info_projection(info: &str) -> SharedString {
    info.to_string().into()
}

fn optimized_info_projection(info: String) -> SharedString {
    info.into()
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    let index = (values.len() - 1) * percentile / 100;
    values[index]
}
