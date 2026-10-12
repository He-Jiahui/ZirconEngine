use std::hint::black_box;
use std::time::Instant;

use super::{strip_wgsl_include_directives, wgsl_include_path_from_line};

const SAMPLE_PAIRS: usize = 21;
const STRIPS_PER_SAMPLE: usize = 2_048;
const LINES_PER_SOURCE: usize = 256;

#[test]
fn optimization_batch_20260826dl_runtime155_include_strip_preserves_line_contract() {
    let source = "\nlet first = 1;\n#include <project::lighting>\n\nlet last = 2;\n";
    assert_eq!(
        strip_wgsl_include_directives(source),
        "\nlet first = 1;\n\nlet last = 2;"
    );
    assert_eq!(
        strip_wgsl_include_directives("#include <project::only>"),
        ""
    );
}

#[test]
fn optimization_batch_20260826dl_runtime155_include_strip_uses_one_output_buffer() {
    let source = include_str!("../module_import.rs");
    let function_start = source
        .find("pub fn strip_wgsl_include_directives")
        .expect("include stripper should remain present");
    let function_tail = &source[function_start..];
    let function_end = function_tail
        .find("\n}\n")
        .expect("include stripper should remain bounded");
    let function = &function_tail[..function_end];

    assert!(function.contains("let mut stripped = String::with_capacity(source.len());"));
    assert!(function.contains("let mut has_output_line = false;"));
    assert!(!function.contains("collect::<Vec<_>>()"));
    assert!(!function.contains(".join(\"\\n\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dl_runtime155_wgsl_include_strip_single_buffer_bench() {
    let source = fixture_source();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&source, legacy_strip));
            optimized_samples.push(measure(&source, strip_wgsl_include_directives));
        } else {
            optimized_samples.push(measure(&source, strip_wgsl_include_directives));
            legacy_samples.push(measure(&source, legacy_strip));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME155_WGSL_INCLUDE_STRIP_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
strips_per_sample={STRIPS_PER_SAMPLE} lines_per_source={LINES_PER_SOURCE} \
legacy_temporary_vecs_per_sample={STRIPS_PER_SAMPLE} optimized_temporary_vecs_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer WGSL include stripping P95 {optimized_p95_ns}ns must be at most 70% of collected-line joining P95 {legacy_p95_ns}ns"
    );
}

fn fixture_source() -> String {
    (0..LINES_PER_SOURCE)
        .map(|index| {
            if index % 4 == 0 {
                format!("#include <project::module_{index:03}>")
            } else {
                format!("let v{index:03}=0;")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn legacy_strip(source: &str) -> String {
    source
        .lines()
        .filter(|line| wgsl_include_path_from_line(line).is_none())
        .collect::<Vec<_>>()
        .join("\n")
}

fn measure(source: &str, strip: fn(&str) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..STRIPS_PER_SAMPLE {
        checksum ^= black_box(strip(black_box(source))).len();
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
