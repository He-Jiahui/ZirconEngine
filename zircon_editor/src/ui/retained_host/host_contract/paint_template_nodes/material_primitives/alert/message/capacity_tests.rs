use std::hint::black_box;
use std::time::Instant;

use super::alert_message;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

const SAMPLE_PAIRS: usize = 21;
const MESSAGES_PER_SAMPLE: usize = 8_192;
const MESSAGE_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826ft_editor161_alert_message_preserves_fallback_order() {
    let node = TemplatePaneNodeData {
        value_text: "value fallback".into(),
        validation_message: "validation fallback".into(),
        options_text: "options fallback".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(alert_message(&node), "value fallback");
}

#[test]
fn optimization_batch_20260826ft_editor161_alert_clone_occurs_after_early_returns() {
    let source = include_str!("../message.rs");
    let borrow = source
        .find("let message = alert_message(node);")
        .expect("borrowed message");
    let frame = source
        .find("let Some((frame, font_size, line_height))")
        .expect("layout early return");
    let clone = source
        .find("message.to_string(),")
        .expect("command-owned message");

    assert!(source.contains("fn alert_message(node: &TemplatePaneNodeData) -> &str"));
    assert!(borrow < frame && frame < clone);
    assert_eq!(source.matches("message.to_string()").count(), 1);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ft_editor161_alert_message_deferred_clone_bench() {
    let message = "a".repeat(MESSAGE_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&message, false));
            optimized_samples.push(measure(&message, true));
        } else {
            optimized_samples.push(measure(&message, true));
            legacy_samples.push(measure(&message, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR161_ALERT_MESSAGE_DEFERRED_CLONE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
messages_per_sample={MESSAGES_PER_SAMPLE} message_bytes={MESSAGE_BYTES} \
legacy_clones_per_rejected_message=1 optimized_clones_per_rejected_message=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(message: &str, defer_clone: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..MESSAGES_PER_SAMPLE {
        let right = black_box(0.0f32);
        let left = black_box(1.0f32);
        if defer_clone {
            let borrowed = black_box(message);
            if right <= left {
                checksum ^= black_box(borrowed.len());
                continue;
            }
            checksum ^= black_box(borrowed.to_string().len());
        } else {
            let owned = black_box(message).to_string();
            if right <= left {
                checksum ^= black_box(owned.len());
                continue;
            }
            checksum ^= black_box(owned.len());
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
