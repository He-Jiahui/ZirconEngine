use std::hint::black_box;
use std::time::Instant;

use super::EntityPath;

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const PATH_SEGMENTS: usize = 1024;

fn legacy_parse(path: &str) -> Result<EntityPath, super::PathParseError> {
    EntityPath::new(
        path.split('/')
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>(),
    )
}

fn measure(path: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut segments = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let parsed = if optimized {
            EntityPath::parse(black_box(path))
        } else {
            legacy_parse(black_box(path))
        };
        if let Ok(parsed) = parsed {
            segments += parsed.segments().len();
        }
    }
    black_box(segments);
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

#[test]
fn runtime_hotpath_batch_runtime340_341_entity_paths_preserve_results() {
    for path in ["root/child", " /root// child/ ", "", " / "] {
        let baseline = legacy_parse(path);
        let candidate = EntityPath::parse(path);
        assert_eq!(
            candidate.map(|value| value.segments().to_vec()),
            baseline.map(|value| value.segments().to_vec()),
            "{path:?}"
        );
    }
}

#[test]
fn runtime_hotpath_batch_runtime340_341_entity_path_reserves_segments() {
    let source = include_str!("../entity_path.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(production.contains("Vec::with_capacity(path.len().saturating_div(2).max(1))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime_hotpath_batch_runtime340_341_preallocated_entity_path_bench() {
    let path = (0..PATH_SEGMENTS)
        .map(|_| "segment")
        .collect::<Vec<_>>()
        .join("/");
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&path, false));
            candidate.push(measure(&path, true));
        } else {
            candidate.push(measure(&path, true));
            baseline.push(measure(&path, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!("RUNTIME340_SINGLE_SCAN_ENTITY_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} path_segments={PATH_SEGMENTS} baseline_vec_reserves=0 candidate_vec_reserves=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}", sample_csv(&baseline), sample_csv(&candidate));
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
