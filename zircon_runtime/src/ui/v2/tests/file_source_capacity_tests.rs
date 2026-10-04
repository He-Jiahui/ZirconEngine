use std::hint::black_box;
use std::time::Instant;

#[test]
fn runtime855_v2_file_source_capacity_preserves_order() {
    let production = include_str!("../file_cache.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production file-cache source");
    let collector = production
        .split("fn collect_v2_sources")
        .nth(1)
        .expect("source collector")
        .split("fn push_source_path")
        .next()
        .expect("collector ends before queue helper");

    assert_eq!(
        collector.matches("Vec::with_capacity(paths.len())").count(),
        2
    );
    assert!(!collector.contains("let mut queue = Vec::new();"));
    assert!(!collector.contains("let mut sources = Vec::new();"));

    let mut queue = Vec::with_capacity(3);
    for path in ["root", "import-a", "import-b"] {
        queue.push(path);
    }
    let mut sources = Vec::with_capacity(3);
    let mut index = 0;
    while index < queue.len() {
        sources.push(queue[index]);
        index += 1;
    }
    assert_eq!(sources, ["root", "import-a", "import-b"]);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime855_v2_file_source_capacity_release_benchmark() {
    const ROOT_PATHS: usize = 4_096;
    const BUILDS_PER_SAMPLE: usize = 1_024;
    const SAMPLE_PAIRS: usize = 17;
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_collection(ROOT_PATHS, BUILDS_PER_SAMPLE, false));
            optimized_ns.push(measure_collection(ROOT_PATHS, BUILDS_PER_SAMPLE, true));
        } else {
            optimized_ns.push(measure_collection(ROOT_PATHS, BUILDS_PER_SAMPLE, true));
            legacy_ns.push(measure_collection(ROOT_PATHS, BUILDS_PER_SAMPLE, false));
        }
    }
    println!(
        "RUNTIME855_V2_FILE_SOURCE_CAPACITY_BENCH_V1 root_paths={ROOT_PATHS} builds_per_sample={BUILDS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_queue_growth_events={} optimized_queue_growth_events=0 legacy_source_growth_events={} optimized_source_growth_events=0 legacy_p50_ns={} legacy_p95_ns={} legacy_p99_ns={} optimized_p50_ns={} optimized_p95_ns={} optimized_p99_ns={} legacy_raw_ns={} optimized_raw_ns={}",
        growth_events(ROOT_PATHS),
        growth_events(ROOT_PATHS),
        percentile(&legacy_ns, 50),
        percentile(&legacy_ns, 95),
        percentile(&legacy_ns, 99),
        percentile(&optimized_ns, 50),
        percentile(&optimized_ns, 95),
        percentile(&optimized_ns, 99),
        csv(&legacy_ns),
        csv(&optimized_ns),
    );
}

fn measure_collection(root_paths: usize, builds: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..builds {
        let mut queue = if optimized {
            Vec::with_capacity(root_paths)
        } else {
            Vec::new()
        };
        for path in 0..root_paths {
            queue.push(path);
        }
        let mut sources = if optimized {
            Vec::with_capacity(root_paths)
        } else {
            Vec::new()
        };
        for path in queue {
            sources.push(path);
        }
        checksum = checksum.wrapping_add(sources.len());
        black_box(sources);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(length: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for index in 1..=length {
        if index > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
