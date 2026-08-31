use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::{layout::UiSize, tree::UiTemplateNodeMetadata};

use super::{MaterialLayoutMetrics, measure_material_content};

const PERF_MARKER: &str = "RUNTIME365_MATERIAL_METRICS_SINGLE_PASS_BENCH_V1";

fn metadata() -> UiTemplateNodeMetadata {
    let mut metadata = UiTemplateNodeMetadata {
        component: "Button".to_string(),
        ..UiTemplateNodeMetadata::default()
    };
    for (key, value) in [
        ("layout_padding_left", 4.0),
        ("layout_padding_right", 5.0),
        ("layout_padding_top", 6.0),
        ("layout_padding_bottom", 7.0),
        ("layout_spacing", 8.0),
        ("layout_min_width", 80.0),
        ("layout_min_height", 24.0),
        ("layout_icon_size", 16.0),
        ("layout_leading_slot_width", 12.0),
        ("layout_trailing_slot_width", 14.0),
    ] {
        metadata
            .attributes
            .insert(key.to_string(), Value::Float(value));
    }
    metadata
}

#[test]
fn optimization_batch_20260830bm_runtime_material_metrics_preserves_results() {
    let metadata = black_box(metadata());
    let measured = measure_material_content(Some(&metadata), UiSize::new(40.0, 12.0))
        .expect("authored material metrics should resolve");
    assert_eq!(measured, UiSize::new(99.0, 29.0));
    assert!(MaterialLayoutMetrics::resolve(&metadata).is_some());
}

#[test]
fn optimization_batch_20260830bm_runtime_material_metrics_source_contract() {
    let source = include_str!("../material.rs");
    assert!(source.contains("for (key, value) in &metadata.attributes"));
    assert!(source.contains("let target = match key.as_str()"));
    assert!(!source.contains("fn number_attr("));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bm_runtime_material_metrics_p95() {
    const MATCHES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let metadata = metadata();
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0.0;
            for _ in 0..MATCHES {
                let metrics = if pass == 0 {
                    let mut authored = false;
                    let mut values = [0.0_f32; 10];
                    for (index, key) in [
                        "layout_padding_left",
                        "layout_padding_right",
                        "layout_padding_top",
                        "layout_padding_bottom",
                        "layout_spacing",
                        "layout_min_width",
                        "layout_min_height",
                        "layout_icon_size",
                        "layout_leading_slot_width",
                        "layout_trailing_slot_width",
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        if let Some(value) = metadata.attributes.get(key) {
                            authored = true;
                            values[index] = value.as_float().unwrap_or_default() as f32;
                        }
                    }
                    authored.then_some(values)
                } else {
                    MaterialLayoutMetrics::resolve(&metadata).map(|metrics| {
                        [
                            metrics.padding_left,
                            metrics.padding_right,
                            metrics.padding_top,
                            metrics.padding_bottom,
                            metrics.spacing,
                            metrics.min_width,
                            metrics.min_height,
                            metrics.icon_size,
                            metrics.leading_slot_width,
                            metrics.trailing_slot_width,
                        ]
                    })
                };
                checksum += metrics.map_or(0.0, |values| values[0] + values[9]);
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
