use std::borrow::Cow;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use super::{project_activation_action, project_activation_summary};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 80_000;
const PROJECT_NAME: &str = "shipping_editor_project_with_a_long_descriptive_name";
const DEFAULT_SCENE_URI: &str = "asset://scenes/shipping/default_editor_scene.zscene";
const DIAGNOSTIC_PATH: &str = r"C:\projects\shipping\.zircon\workspace-layout.json";
const DIAGNOSTIC_MESSAGE: &str =
    "workspace layout schema was obsolete and the default layout was restored";

#[test]
fn optimization_batch_20260829aj_editor255_activation_status_preserves_exact_text() {
    let action = project_activation_action("Project opened", 64, 64, 0, "persisted-v1");
    assert_eq!(
        project_activation_summary(
            &action,
            PROJECT_NAME,
            DEFAULT_SCENE_URI,
            64,
            64,
            0,
            2,
            "persisted-v1",
            None,
        ),
        legacy_activation_status(false)
    );
    assert_eq!(
        optimized_activation_status(),
        legacy_activation_status(true)
    );
}

#[test]
fn optimization_batch_20260829aj_editor255_healthy_action_and_summary_avoid_intermediate_buffers() {
    assert!(matches!(
        project_activation_action("Project opened", 64, 64, 0, "persisted-v1"),
        Cow::Borrowed("Project opened")
    ));

    let source = include_str!("../create_or_open.rs");
    let status_builder = source
        .split("fn project_activation_status_message")
        .nth(1)
        .expect("project activation status builder")
        .split("fn project_activation_summary")
        .next()
        .expect("project activation status builder body");
    assert!(status_builder.contains("project_activation_summary("));
    assert!(!status_builder.contains("let summary = format!("));
    assert!(!status_builder.contains("format!(\n        \"{summary}"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829aj_editor255_single_buffer_activation_status_bench() {
    assert_eq!(
        optimized_activation_status(),
        legacy_activation_status(true)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR255_SINGLE_BUFFER_PROJECT_ACTIVATION_STATUS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} legacy_string_allocations_per_build=3 \
optimized_string_allocations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_activation_status(with_diagnostic: bool) -> String {
    let action = "Project opened".to_string();
    let summary = format!(
        "{action}: {PROJECT_NAME} (scene={DEFAULT_SCENE_URI} assets=64 ready=64 failed=0 registry_diagnostics=2 project_settings=persisted-v1)"
    );
    if !with_diagnostic {
        return summary;
    }
    format!(
        "{summary}; using default layout after workspace restore failed from {DIAGNOSTIC_PATH}: {DIAGNOSTIC_MESSAGE}"
    )
}

fn optimized_activation_status() -> String {
    let action = project_activation_action("Project opened", 64, 64, 0, "persisted-v1");
    project_activation_summary(
        &action,
        PROJECT_NAME,
        DEFAULT_SCENE_URI,
        64,
        64,
        0,
        2,
        "persisted-v1",
        Some((Path::new(DIAGNOSTIC_PATH), DIAGNOSTIC_MESSAGE)),
    )
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let status = if optimized {
            optimized_activation_status()
        } else {
            legacy_activation_status(true)
        };
        checksum = checksum.wrapping_add(black_box(status).len());
    }
    black_box(checksum);
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
