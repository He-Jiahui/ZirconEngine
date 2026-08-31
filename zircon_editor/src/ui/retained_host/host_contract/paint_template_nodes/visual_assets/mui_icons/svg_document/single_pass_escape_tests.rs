use std::hint::black_box;
use std::time::Instant;

use super::escape_xml_attribute;

const SAMPLE_PAIRS: usize = 31;
const ESCAPES_PER_SAMPLE: usize = 40_000;

#[test]
fn optimization_batch_20260829y_editor244_attribute_escape_preserves_bytes() {
    for (input, expected) in [
        ("", ""),
        ("plain-value", "plain-value"),
        (
            "alpha&\"'<beta>\u{96ea}",
            "alpha&amp;&quot;'&lt;beta&gt;\u{96ea}",
        ),
        ("&&<<\"\">>", "&amp;&amp;&lt;&lt;&quot;&quot;&gt;&gt;"),
    ] {
        assert_eq!(escape_xml_attribute(input), expected);
        assert_eq!(
            escape_xml_attribute(input),
            legacy_escape_xml_attribute(input)
        );
    }
}

#[test]
fn optimization_batch_20260829y_editor244_attribute_escape_uses_one_buffer() {
    let source = include_str!("../svg_document.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("fn escape_xml_attribute")
        .nth(1)
        .expect("attribute escape helper");

    assert!(body.contains("String::with_capacity"));
    assert!(body.contains("escaped.push_str"));
    assert!(!body.contains(".replace("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829y_editor244_single_pass_svg_attribute_escape_bench() {
    let input = "M2&3<5 \"retained-host\" data=\u{96ea}> repeat&&<<\"\">> generation-safe";
    assert_eq!(
        escape_xml_attribute(input),
        legacy_escape_xml_attribute(input)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, input));
            optimized_samples.push(measure(true, input));
        } else {
            optimized_samples.push(measure(true, input));
            legacy_samples.push(measure(false, input));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR244_SINGLE_PASS_SVG_ATTRIBUTE_ESCAPE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
escapes_per_sample={ESCAPES_PER_SAMPLE} input_bytes={} \
legacy_result_allocations_per_escape=4 optimized_result_allocations_per_escape=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        input.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_escape_xml_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn measure(optimized: bool, input: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..ESCAPES_PER_SAMPLE {
        let escaped = if optimized {
            escape_xml_attribute(black_box(input))
        } else {
            legacy_escape_xml_attribute(black_box(input))
        };
        checksum = checksum.wrapping_add(black_box(escaped).len());
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
