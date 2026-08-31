use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::surface::UiNavigationEventKind;

use super::logical_directional_navigation_kind;

const CHECKS_PER_SAMPLE: usize = 262_144;
const SAMPLE_PAIRS: usize = 31;

fn legacy_matches_any(key: &str, expected: &[&str]) -> bool {
    expected.iter().any(|expected| {
        key.bytes()
            .filter(u8::is_ascii_alphanumeric)
            .map(|byte| byte.to_ascii_lowercase())
            .eq(expected.bytes())
    })
}

fn legacy_logical_directional_navigation_kind(logical_key: &str) -> Option<UiNavigationEventKind> {
    if legacy_matches_any(logical_key, &["arrowleft", "left", "gamepaddpadleft"]) {
        Some(UiNavigationEventKind::Left)
    } else if legacy_matches_any(logical_key, &["arrowup", "up", "gamepaddpadup"]) {
        Some(UiNavigationEventKind::Up)
    } else if legacy_matches_any(logical_key, &["arrowright", "right", "gamepaddpadright"]) {
        Some(UiNavigationEventKind::Right)
    } else if legacy_matches_any(logical_key, &["arrowdown", "down", "gamepaddpaddown"]) {
        Some(UiNavigationEventKind::Down)
    } else {
        None
    }
}

fn measure(key: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut matches = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        matches += usize::from(if optimized {
            logical_directional_navigation_kind(black_box(key)).is_some()
        } else {
            legacy_logical_directional_navigation_kind(black_box(key)).is_some()
        });
    }
    black_box(matches);
    started.elapsed().as_nanos().max(1)
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

#[test]
fn runtime_hotpath_batch_runtime337_338_direction_keys_preserve_results() {
    for key in [
        "ArrowLeft",
        "arrow-left",
        "GAMEPAD_DPAD_RIGHT",
        "Arrow Up",
        "gamepad/dpad/down",
        "leftover",
        "gamepaddpadrightx",
        "",
        "\u{2192}",
    ] {
        assert_eq!(
            logical_directional_navigation_kind(key),
            legacy_logical_directional_navigation_kind(key),
            "{key:?}"
        );
    }
}

#[test]
fn runtime_hotpath_batch_runtime337_338_direction_key_normalizes_once() {
    let source = include_str!("../keyboard_navigation.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert!(production.contains("MAX_NORMALIZED_DIRECTION_KEY_BYTES"));
    assert!(production.contains("let mut normalized = [0; MAX_NORMALIZED_DIRECTION_KEY_BYTES]"));
    assert!(!production.contains("fn normalized_key_matches_any"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime_hotpath_batch_runtime337_338_single_normalize_direction_key_bench() {
    let key = "gamepaddpadrightx";
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(key, false));
            candidate_samples.push(measure(key, true));
        } else {
            candidate_samples.push(measure(key, true));
            baseline_samples.push(measure(key, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "RUNTIME338_SINGLE_NORMALIZE_DIRECTION_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} key_bytes={} \
baseline_normalization_passes=12 candidate_normalization_passes=1 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        key.len(),
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
