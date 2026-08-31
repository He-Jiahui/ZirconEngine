use std::hint::black_box;
use std::time::Instant;

use super::prefixed_pascal_class;

const SAMPLE_PAIRS: usize = 31;
const CLASSES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829ae_runtime304_pascal_classes_preserve_bytes() {
    for (prefix, infix, value, expected) in [
        (
            "MuiDataGrid",
            "-root--density",
            "compact",
            "MuiDataGrid-root--densityCompact",
        ),
        (
            "MuiDataGrid",
            "-rowSpacing",
            "border_top",
            "MuiDataGrid-rowSpacingBorderTop",
        ),
        (
            "MuiDataGrid",
            "-sortingMode",
            "server side",
            "MuiDataGrid-sortingModeServerSide",
        ),
        (
            "MuiDataGrid",
            "-",
            "noRowsOverlay",
            "MuiDataGrid-NoRowsOverlay",
        ),
        (
            "MuiDataGrid",
            "-",
            "--snow__\u{96ea}",
            "MuiDataGrid-Snow\u{96ea}",
        ),
    ] {
        assert_eq!(prefixed_pascal_class(prefix, infix, value), expected);
        assert_eq!(
            prefixed_pascal_class(prefix, infix, value),
            legacy_prefixed_pascal_class(prefix, infix, value)
        );
    }
}

#[test]
fn optimization_batch_20260829ae_runtime304_pascal_classes_use_one_buffer() {
    let source = include_str!("../data_grid.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let builder = implementation
        .split("fn prefixed_pascal_class")
        .nth(1)
        .expect("single-buffer class builder");

    assert!(builder.contains("String::with_capacity"));
    assert!(builder.contains("class.push_str(prefix)"));
    assert!(builder.contains("class.push_str(infix)"));
    assert!(builder.contains("class.push("));
    assert!(!builder.contains("format!("));
    assert!(!builder.contains("collect::<String>"));
    assert_eq!(source.matches("prefixed_pascal_class(").count(), 7);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ae_runtime304_single_buffer_pascal_class_bench() {
    let prefix = "MuiDataGridPremium";
    let infix = "-root--density";
    let value = "server_side row-selection__animation-review mode";
    assert_eq!(
        prefixed_pascal_class(prefix, infix, value),
        legacy_prefixed_pascal_class(prefix, infix, value)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, prefix, infix, value));
            optimized_samples.push(measure(true, prefix, infix, value));
        } else {
            optimized_samples.push(measure(true, prefix, infix, value));
            legacy_samples.push(measure(false, prefix, infix, value));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME304_SINGLE_BUFFER_DATA_GRID_PASCAL_CLASS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
classes_per_sample={CLASSES_PER_SAMPLE} value_bytes={} \
legacy_result_buffers_per_class=2 optimized_result_buffers_per_class=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        value.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_prefixed_pascal_class(prefix: &str, infix: &str, value: &str) -> String {
    format!("{prefix}{infix}{}", legacy_pascal_case(value))
}

fn legacy_pascal_case(value: &str) -> String {
    value
        .split(['-', '_', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn measure(optimized: bool, prefix: &str, infix: &str, value: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CLASSES_PER_SAMPLE {
        let class = if optimized {
            prefixed_pascal_class(black_box(prefix), black_box(infix), black_box(value))
        } else {
            legacy_prefixed_pascal_class(black_box(prefix), black_box(infix), black_box(value))
        };
        checksum = checksum.wrapping_add(black_box(class).len());
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
