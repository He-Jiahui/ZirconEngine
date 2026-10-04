use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const DEPENDENCY_BYTES: usize = 2_048;
const PHYSICAL_PATH_BYTES: usize = 192;
const INSERTS_PER_SAMPLE: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_ft_editor406_reuses_duplicate_dependency_key() {
    let mut dependencies = BTreeSet::new();
    insert_dependency(&mut dependencies, "res://ui/shared.zui");
    insert_dependency(&mut dependencies, "res://ui/shared.zui");

    assert_eq!(dependencies.len(), 1);
    assert!(dependencies.contains("res://ui/shared.zui"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ft_editor406_borrowed_duplicate_dependency_benchmark() {
    let dependency = format!("res://ui/{}.zui", "d".repeat(DEPENDENCY_BYTES - 13));
    for _ in 0..4 {
        black_box(measure_duplicate_inserts(&dependency, false));
        black_box(measure_duplicate_inserts(&dependency, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_duplicate_inserts(&dependency, false));
            optimized_samples.push(measure_duplicate_inserts(&dependency, true));
        } else {
            optimized_samples.push(measure_duplicate_inserts(&dependency, true));
            legacy_samples.push(measure_duplicate_inserts(&dependency, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR406_BORROWED_DUPLICATE_IMPORT_DEPENDENCY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} inserts_per_sample={INSERTS_PER_SAMPLE} dependency_bytes={} legacy_owned_keys_per_sample={INSERTS_PER_SAMPLE} optimized_owned_keys_per_sample=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
        dependency.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 65 / 100);
}

#[test]
#[ignore = "release performance gate"]
fn editor406_borrowed_duplicate_physical_path_benchmark() {
    let path = physical_path_fixture(0);
    for _ in 0..4 {
        black_box(measure_duplicate_path_inserts(&path, false));
        black_box(measure_duplicate_path_inserts(&path, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_duplicate_path_inserts(&path, false));
            optimized_samples.push(measure_duplicate_path_inserts(&path, true));
        } else {
            optimized_samples.push(measure_duplicate_path_inserts(&path, true));
            legacy_samples.push(measure_duplicate_path_inserts(&path, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR406_BORROWED_DUPLICATE_PHYSICAL_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} inserts_per_sample={INSERTS_PER_SAMPLE} path_bytes={PHYSICAL_PATH_BYTES} legacy_owned_paths_per_sample={INSERTS_PER_SAMPLE} optimized_owned_paths_per_sample=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 65 / 100);
}

#[test]
#[ignore = "release performance non-regression gates"]
fn editor406_physical_path_unique_and_mixed_benchmark() {
    const EVENT_COUNT: usize = 128;
    let unique = (0..EVENT_COUNT)
        .map(physical_path_fixture)
        .collect::<Vec<_>>();
    let mixed = (0..EVENT_COUNT)
        .map(|index| physical_path_fixture(index % (EVENT_COUNT / 2)))
        .collect::<Vec<_>>();
    assert_eq!(unique.iter().collect::<BTreeSet<_>>().len(), EVENT_COUNT);
    assert_eq!(mixed.iter().collect::<BTreeSet<_>>().len(), EVENT_COUNT / 2);
    let unique_p95 = benchmark_path_workload("all_unique", &unique);
    let mixed_p95 = benchmark_path_workload("mixed_50_percent_unique", &mixed);
    for (workload, (legacy_p95, optimized_p95)) in [
        ("all_unique", unique_p95),
        ("mixed_50_percent_unique", mixed_p95),
    ] {
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
            "{workload} borrowed path p95 {optimized_p95}ns exceeded 110% of owned path baseline {legacy_p95}ns"
        );
    }
}

fn physical_path_fixture(index: usize) -> PathBuf {
    const PREFIX: &str = "C:/assets/ui/components/";
    let suffix = format!("_{index:04}.zui");
    let path = PathBuf::from(format!(
        "{PREFIX}{}{suffix}",
        "p".repeat(PHYSICAL_PATH_BYTES - PREFIX.len() - suffix.len())
    ));
    assert_eq!(path.to_string_lossy().len(), PHYSICAL_PATH_BYTES);
    assert!(path
        .components()
        .all(|component| component.as_os_str().to_string_lossy().len() <= 255));
    path
}

fn benchmark_path_workload(workload: &str, paths: &[PathBuf]) -> (u128, u128) {
    const ROUNDS_PER_SAMPLE: usize = 128;
    for _ in 0..2 {
        black_box(measure_path_batch(paths, ROUNDS_PER_SAMPLE, false));
        black_box(measure_path_batch(paths, ROUNDS_PER_SAMPLE, true));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_path_batch(paths, ROUNDS_PER_SAMPLE, false));
            optimized_samples.push(measure_path_batch(paths, ROUNDS_PER_SAMPLE, true));
        } else {
            optimized_samples.push(measure_path_batch(paths, ROUNDS_PER_SAMPLE, true));
            legacy_samples.push(measure_path_batch(paths, ROUNDS_PER_SAMPLE, false));
        }
    }
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR406_PHYSICAL_PATH_UNIQUE_MIXED_BENCH_V1 workload={workload} path_bytes={PHYSICAL_PATH_BYTES} events_per_round={} rounds_per_sample={ROUNDS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={} legacy_p95_ns={legacy_p95} legacy_p99_ns={} optimized_p50_ns={} optimized_p95_ns={optimized_p95} optimized_p99_ns={} threshold_p95_ratio=1.10",
        paths.len(),
        percentile(&legacy_samples, 50),
        percentile(&legacy_samples, 99),
        percentile(&optimized_samples, 50),
        percentile(&optimized_samples, 99),
    );
    (legacy_p95, optimized_p95)
}

fn measure_path_batch(paths: &[PathBuf], rounds: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..rounds {
        let mut expanded = BTreeSet::<PathBuf>::new();
        for path in paths {
            if optimized {
                black_box(mark_physical_path_expanded(&mut expanded, black_box(path)));
            } else {
                black_box(expanded.insert(black_box(path).to_path_buf()));
            }
        }
        black_box(expanded);
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_duplicate_inserts(dependency: &str, optimized: bool) -> u128 {
    let mut dependencies = BTreeSet::new();
    let started = Instant::now();
    for _ in 0..INSERTS_PER_SAMPLE {
        if optimized {
            insert_dependency(black_box(&mut dependencies), black_box(dependency));
        } else {
            dependencies.insert(black_box(dependency).to_owned());
        }
    }
    black_box(dependencies);
    started.elapsed().as_nanos().max(1)
}

fn measure_duplicate_path_inserts(path: &Path, optimized: bool) -> u128 {
    let mut expanded = BTreeSet::<PathBuf>::new();
    let started = Instant::now();
    for _ in 0..INSERTS_PER_SAMPLE {
        if optimized {
            black_box(mark_physical_path_expanded(&mut expanded, black_box(path)));
        } else {
            black_box(expanded.insert(black_box(path).to_path_buf()));
        }
    }
    black_box(expanded);
    started.elapsed().as_nanos().max(1)
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
