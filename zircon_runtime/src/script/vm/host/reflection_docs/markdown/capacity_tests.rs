use std::hint::black_box;
use std::time::Instant;

use super::{capability_line_capacity, push_capability_line};

const SAMPLE_PAIRS: usize = 21;
const LINES_PER_SAMPLE: usize = 1_024;
const CAPABILITIES_PER_LINE: usize = 64;

#[test]
fn optimization_batch_20260826fq_runtime212_direct_render_preserves_capability_line() {
    let capabilities = ["asset.read", "render.submit", "scene.write"];
    let label = "- Required capabilities: ";
    let mut output = String::new();

    push_capability_line(&mut output, label, &capabilities);

    assert_eq!(
        output,
        "- Required capabilities: `asset.read`, `render.submit`, `scene.write`\n"
    );
    assert!(output.capacity() >= capability_line_capacity(label, &capabilities));
    let mut empty = String::new();
    push_capability_line(&mut empty, "- Capabilities: ", &[]);
    assert_eq!(empty, "- Capabilities: \n");
}

#[test]
fn optimization_batch_20260826fq_runtime212_capabilities_write_without_temporary_strings() {
    let source = include_str!("../markdown.rs");
    assert!(source.contains("push_capability_line(output, label, &capabilities);"));
    assert!(source.contains("output.reserve(capability_line_capacity(label, capabilities));"));
    assert!(!source.contains(".map(|capability| format!(\"`{capability}`\"))"));
    assert!(!source.contains("push_line(output, &format!(\"{label}{capabilities}\"));"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fq_runtime212_reflection_capability_direct_render_bench() {
    let owned = (0..CAPABILITIES_PER_LINE)
        .map(|index| format!("runtime.capability.{index:04}"))
        .collect::<Vec<_>>();
    let capabilities = owned.iter().map(String::as_str).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&capabilities, false));
            optimized_samples.push(measure(&capabilities, true));
        } else {
            optimized_samples.push(measure(&capabilities, true));
            legacy_samples.push(measure(&capabilities, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME212_REFLECTION_CAPABILITY_DIRECT_RENDER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lines_per_sample={LINES_PER_SAMPLE} capabilities_per_line={CAPABILITIES_PER_LINE} \
legacy_temporary_strings_per_line={} optimized_temporary_strings_per_line=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        CAPABILITIES_PER_LINE + 2,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(capabilities: &[&str], direct: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LINES_PER_SAMPLE {
        let mut output = String::new();
        if direct {
            push_capability_line(&mut output, "- Capabilities: ", black_box(capabilities));
        } else {
            legacy_push_capability_line(&mut output, "- Capabilities: ", black_box(capabilities));
        }
        checksum ^= black_box(output.len() ^ output.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_push_capability_line(output: &mut String, label: &str, capabilities: &[&str]) {
    let capabilities = capabilities
        .iter()
        .map(|capability| format!("`{capability}`"))
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str(&format!("{label}{capabilities}"));
    output.push('\n');
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
