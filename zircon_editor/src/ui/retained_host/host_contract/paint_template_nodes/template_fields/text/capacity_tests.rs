use std::hint::black_box;
use std::time::Instant;

use super::field_label;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

const SAMPLE_PAIRS: usize = 21;
const FIELDS_PER_SAMPLE: usize = 8_192;
const LABEL_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826fx_editor165_field_label_preserves_disabled_fallback() {
    let disabled = TemplatePaneNodeData {
        control_id: "WorkbenchInputDisabled".into(),
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(field_label(&disabled), "Disabled input");

    let empty = TemplatePaneNodeData::default();
    assert!(field_label(&empty).is_empty());
}

#[test]
fn optimization_batch_20260826fx_editor165_field_label_projects_after_frame_gate() {
    let source = include_str!("../text.rs");
    let frame_gate = source
        .find("if !frame_is_within(&text_rect, rect)")
        .expect("frame gate");
    let label_projection = source
        .find("let label = field_label(node);")
        .expect("label projection");
    let command = source
        .find("commands.push(HostPaintCommand::text(")
        .expect("text command");

    assert!(frame_gate < label_projection && label_projection < command);
    assert_eq!(source.matches("let label = field_label(node);").count(), 1);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fx_editor165_field_label_deferred_projection_bench() {
    let label = "f".repeat(LABEL_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&label, false));
            optimized_samples.push(measure(&label, true));
        } else {
            optimized_samples.push(measure(&label, true));
            legacy_samples.push(measure(&label, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR165_FIELD_LABEL_DEFERRED_PROJECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
fields_per_sample={FIELDS_PER_SAMPLE} label_bytes={LABEL_BYTES} \
legacy_projections_per_rejected_field=1 optimized_projections_per_rejected_field=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(label: &str, defer_projection: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..FIELDS_PER_SAMPLE {
        let frame_is_valid = black_box(false);
        if defer_projection {
            if !frame_is_valid {
                checksum ^= black_box(label.len());
                continue;
            }
            checksum ^= black_box(label.to_string().len());
        } else {
            let projected = black_box(label).to_string();
            if !frame_is_valid {
                checksum ^= black_box(projected.len());
                continue;
            }
            checksum ^= black_box(projected.len());
        }
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
