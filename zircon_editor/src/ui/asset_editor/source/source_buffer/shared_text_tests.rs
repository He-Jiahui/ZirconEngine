use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::*;

const SOURCE_BYTES: usize = 256 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hp_editor208_shares_clean_source_and_saved_text() {
    let mut buffer = UiAssetSourceBuffer::new("initial source");

    assert!(Arc::ptr_eq(&buffer.text, &buffer.saved_text));
    assert!(!buffer.is_dirty());

    buffer.replace("edited source");
    assert!(!Arc::ptr_eq(&buffer.text, &buffer.saved_text));
    assert!(buffer.is_dirty());
    assert_eq!(buffer.text(), "edited source");

    buffer.mark_saved();
    assert!(Arc::ptr_eq(&buffer.text, &buffer.saved_text));
    assert!(!buffer.is_dirty());
}

#[test]
fn optimization_batch_20260826hp_editor208_uses_arc_string_without_content_clone() {
    let source = include_str!("../source_buffer.rs");

    assert!(source.contains("text: Arc<String>"));
    assert!(source.contains("saved_text: Arc<String>"));
    assert!(source.contains("saved_text: Arc::clone(&text)"));
    assert!(source.contains("self.saved_text = Arc::clone(&self.text)"));
    assert!(!source.contains("saved_text: text.clone()"));
    assert!(!source.contains("self.saved_text = self.text.clone()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hp_editor208_shared_source_buffer_text_release_benchmark() {
    let source = Arc::new("x".repeat(SOURCE_BYTES));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_saved_text(black_box(source.as_ref())));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(shared_saved_text(black_box(&source)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR208_SHARED_SOURCE_BUFFER_TEXT_BENCH_V1 \
         source_bytes={SOURCE_BYTES} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_saved_text(source: &String) -> String {
    source.clone()
}

fn shared_saved_text(source: &Arc<String>) -> Arc<String> {
    Arc::clone(source)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
