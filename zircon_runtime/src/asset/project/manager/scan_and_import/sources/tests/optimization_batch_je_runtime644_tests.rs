use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

const SOURCE_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_je_runtime644_hashes_project_uri_duplicate_detection() {
    let source = include_str!("../../sources.rs");
    let rejection = source
        .split("fn reject_duplicate_project_uris")
        .nth(1)
        .expect("project URI duplicate rejection remains present")
        .split("fn compound_root_for_meta_path")
        .next()
        .expect("project URI duplicate rejection remains bounded");

    assert!(rejection.contains("HashMap::with_capacity(sources.len())"));
    assert!(!rejection.contains("BTreeMap::new()"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_je_runtime644_hashed_project_uri_duplicate_benchmark() {
    let sources = (0..SOURCE_COUNT)
        .map(|index| (format!("res://batch-644/{index:08}.zasset"), index))
        .collect::<Vec<_>>();
    assert_eq!(ordered_duplicate(&sources), hashed_duplicate(&sources));

    for _ in 0..4 {
        black_box(measure_duplicate_scan(&sources, false));
        black_box(measure_duplicate_scan(&sources, true));
    }

    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hashed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            ordered_samples.push(measure_duplicate_scan(&sources, false));
            hashed_samples.push(measure_duplicate_scan(&sources, true));
        } else {
            hashed_samples.push(measure_duplicate_scan(&sources, true));
            ordered_samples.push(measure_duplicate_scan(&sources, false));
        }
    }

    let ordered_p95 = percentile(&ordered_samples, 95);
    let hashed_p95 = percentile(&hashed_samples, 95);
    let improvement_percent =
        ordered_p95.saturating_sub(hashed_p95).saturating_mul(100) / ordered_p95.max(1);
    println!(
        "RUNTIME644_HASHED_PROJECT_URI_DUPLICATE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} source_count={SOURCE_COUNT} ordered_ns={} hashed_ns={} ordered_p95_ns={ordered_p95} hashed_p95_ns={hashed_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&ordered_samples),
        csv(&hashed_samples),
    );
    assert!(hashed_p95 <= ordered_p95 * 80 / 100);
}

fn measure_duplicate_scan(sources: &[(String, usize)], hashed: bool) -> u128 {
    let started = Instant::now();
    let duplicate = if hashed {
        hashed_duplicate(sources)
    } else {
        ordered_duplicate(sources)
    };
    black_box(duplicate);
    started.elapsed().as_nanos().max(1)
}

fn ordered_duplicate(sources: &[(String, usize)]) -> Option<(usize, usize)> {
    let mut paths_by_uri = BTreeMap::new();
    for (uri, path) in sources {
        if let Some(previous) = paths_by_uri.insert(uri.clone(), *path) {
            return Some((previous, *path));
        }
    }
    None
}

fn hashed_duplicate(sources: &[(String, usize)]) -> Option<(usize, usize)> {
    let mut paths_by_uri = HashMap::with_capacity(sources.len());
    for (uri, path) in sources {
        if let Some(previous) = paths_by_uri.insert(uri.clone(), *path) {
            return Some((previous, *path));
        }
    }
    None
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
