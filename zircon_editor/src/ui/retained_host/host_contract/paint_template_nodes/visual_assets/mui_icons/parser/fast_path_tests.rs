use std::hint::black_box;
use std::time::Instant;

use super::parse_js_double_quoted_value;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 512;
const PATH_BYTES: usize = 4_096;

#[test]
fn optimization_batch_20260826fl_editor153_fast_path_and_escape_fallback_preserve_values() {
    let plain_path = "M".repeat(PATH_BYTES);
    let plain_source = format!("{plain_path}\"tail");
    let (plain, plain_end) = parse_js_double_quoted_value(&plain_source, 0)
        .expect("plain path should use the unescaped value fast path");
    assert_eq!(plain, plain_path);
    assert_eq!(&plain_source[plain_end..], "tail");
    assert!(plain.capacity() >= PATH_BYTES);

    let escaped_source = r#"M1 \"quoted\" z"tail"#;
    let (escaped, escaped_end) = parse_js_double_quoted_value(escaped_source, 0)
        .expect("escaped path should fall back to the decoder");
    assert_eq!(escaped, "M1 \"quoted\" z");
    assert_eq!(&escaped_source[escaped_end..], "tail");
}

#[test]
fn optimization_batch_20260826fl_editor153_parser_routes_plain_values_through_bulk_copy() {
    let source = include_str!("../parser.rs");
    assert!(source.contains("fn unescaped_js_double_quoted_value("));
    assert!(source.contains("if let Some(value) = unescaped_js_double_quoted_value(source, start)"));
    assert!(source.contains("tail[..value_end].to_owned()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fl_editor153_mui_icon_path_fast_path_bench() {
    let source = format!("{}\"", "M".repeat(PATH_BYTES));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&source, false));
            optimized_samples.push(measure(&source, true));
        } else {
            optimized_samples.push(measure(&source, true));
            legacy_samples.push(measure(&source, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR153_MUI_ICON_PATH_FAST_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} path_bytes={PATH_BYTES} \
legacy_byte_pushes_per_build={PATH_BYTES} optimized_bulk_copies_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(source: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let (value, end) = if optimized {
            parse_js_double_quoted_value(black_box(source), 0).expect("benchmark path must parse")
        } else {
            legacy_parse_js_double_quoted_value(black_box(source), 0)
                .expect("benchmark path must parse")
        };
        checksum ^= black_box(value.len() ^ value.capacity() ^ end);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_parse_js_double_quoted_value(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    let mut out = String::new();
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Some((out, index + 1)),
            byte => out.push(byte as char),
        }
        index += 1;
    }
    None
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
