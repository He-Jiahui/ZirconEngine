use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::normalized_asset_relative_path;

const SAMPLE_PAIRS: usize = 21;
const OPERATIONS_PER_SAMPLE: usize = 8_192;

#[test]
fn optimization_batch_20260826hc_editor195_preserves_asset_relative_path_contract() {
    assert_eq!(
        normalized_asset_relative_path("  res://assets/ui/icons/add.svg  "),
        PathBuf::from("ui/icons/add.svg")
    );
    assert_eq!(
        normalized_asset_relative_path(r"res:\\assets\\ui\\icons\\add.svg"),
        PathBuf::from("ui/icons/add.svg")
    );
    assert_eq!(
        normalized_asset_relative_path("res://assets/ui/../icons/add.svg"),
        PathBuf::from("ui/icons/add.svg")
    );
}

#[test]
fn optimization_batch_20260826hc_editor195_borrows_forward_slash_paths() {
    let source = include_str!("../paths.rs");
    let start = source
        .find("fn normalized_asset_relative_path(")
        .expect("normalized_asset_relative_path function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let body = &source[start..end];

    assert!(body.contains("Cow::Borrowed"));
    assert!(body.contains("Cow::Owned"));
    assert!(!body.contains("source.trim().replace"));
    assert!(!body.contains("stripped.to_string()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hc_editor195_asset_path_borrowed_normalization_release_benchmark() {
    let source = format!(
        "res://assets/{}/main.svg",
        (0..96)
            .map(|index| format!("package-{index:03}"))
            .collect::<Vec<_>>()
            .join("/")
    );
    assert_eq!(
        normalized_asset_relative_path(&source),
        legacy_normalized_asset_relative_path(&source)
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_normalized_asset_relative_path(black_box(&source)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(normalized_asset_relative_path(black_box(&source)));
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
        "EDITOR195_ASSET_PATH_BORROWED_NORMALIZATION_BENCH_V1 input_bytes={} \
         operations_per_sample={OPERATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
         legacy_ns={} optimized_ns={}",
        source.len(),
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_normalized_asset_relative_path(source: &str) -> PathBuf {
    let mut value = source.trim().replace('\\', "/");
    if let Some(stripped) = value.strip_prefix("res://") {
        value = stripped.to_string();
    }
    let mut relative = PathBuf::new();
    for component in std::path::Path::new(value.trim_start_matches('/')).components() {
        match component {
            std::path::Component::Prefix(_)
            | std::path::Component::RootDir
            | std::path::Component::CurDir
            | std::path::Component::ParentDir => {}
            std::path::Component::Normal(value)
                if relative.as_os_str().is_empty() && value == std::ffi::OsStr::new("assets") => {}
            std::path::Component::Normal(value) => relative.push(value),
        }
    }
    relative
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
