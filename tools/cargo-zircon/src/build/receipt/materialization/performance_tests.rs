use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::hint::black_box;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

#[test]
#[ignore = "release-only performance evidence"]
fn direct_relative_path_materialization_performance_evidence() {
    const PATH_COUNT: usize = 20_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 80;

    let root = Path::new(r"C:\artifact-root");
    let paths = (0..PATH_COUNT)
        .map(|index| {
            root.join(format!(
                "role_{}/products/shard_{:04}/artifact_{index:06}.dll",
                index % 4,
                index % 1_024
            ))
        })
        .collect::<Vec<_>>();
    let inventory_entries = paths
        .iter()
        .map(|path| {
            let relative = legacy_relative_path(root, path);
            let (directory, file_name) = relative.rsplit_once('/').unwrap();
            (directory.to_string(), OsString::from(file_name))
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_relative_paths(root, &paths, &inventory_entries, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_relative_paths(root, &paths, &inventory_entries, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_MATERIALIZATION_DIRECT_PATH_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 20%"
    );
}

fn measure_relative_paths(
    root: &Path,
    paths: &[PathBuf],
    inventory_entries: &[(String, OsString)],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline = (0_usize, Duration::ZERO);
    let mut candidate = (0_usize, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        baseline.0 = paths
            .iter()
            .map(|path| legacy_relative_path(root, black_box(path)).len())
            .sum();
        baseline.1 = started.elapsed();
    };
    let mut run_candidate = || {
        let started = Instant::now();
        let mut relative = String::new();
        let mut total_length = 0_usize;
        for (directory, file_name) in inventory_entries {
            super::inventory_relative_path_into(
                &mut relative,
                black_box(directory),
                black_box(file_name.as_os_str()),
            )
            .unwrap();
            total_length = total_length.saturating_add(black_box(relative.len()));
        }
        candidate.0 = total_length;
        candidate.1 = started.elapsed();
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

fn legacy_relative_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap();
    let mut components = Vec::new();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            panic!("fixture path must be normalized");
        };
        components.push(component.to_str().unwrap());
    }
    components.join("/")
}

#[test]
#[ignore = "release-only performance evidence"]
fn materialization_path_buffer_performance_evidence() {
    const PATH_COUNT: usize = 20_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 80;

    let root = Path::new(r"C:\artifact-root");
    let paths = (0..PATH_COUNT)
        .map(|index| {
            format!(
                "role_{}/products/shard_{:04}/artifact_{index:06}.dll",
                index % 4,
                index % 1_024
            )
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_materialization_paths(root, &paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_materialization_paths(root, &paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_MATERIALIZATION_PATH_BUFFER_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 20%"
    );
}

fn measure_materialization_paths(
    root: &Path,
    paths: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let length = paths
            .iter()
            .map(|relative| root.join(black_box(relative)).as_os_str().len())
            .sum::<usize>();
        (black_box(length), started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let mut path = PathBuf::new();
        let mut length = 0_usize;
        for relative in paths {
            super::materialization_path_into(&mut path, root, black_box(relative));
            length = length.saturating_add(black_box(path.as_os_str().len()));
        }
        (length, started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn consuming_inventory_membership_performance_evidence() {
    const PATH_COUNT: usize = 50_000;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 75;

    let paths = (0..PATH_COUNT)
        .map(|index| {
            format!(
                "role_{}/products/shard_{:04}/artifact_{index:06}.dll",
                index % 4,
                index % 1_024
            )
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_inventories(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }

    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_inventories(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_MATERIALIZATION_CONSUMING_INVENTORY_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 25%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 25%"
    );
}

fn measure_inventories(
    paths: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline = (0_usize, Duration::ZERO);
    let mut candidate = (0_usize, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        let actual = paths.iter().cloned().collect::<BTreeSet<_>>();
        let expected = paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
        assert!(
            actual.iter().all(|path| expected.contains(path.as_str()))
                && expected.iter().all(|path| actual.contains(*path))
        );
        baseline = (actual.len(), started.elapsed());
    };
    let mut run_candidate = || {
        let started = Instant::now();
        let mut expected = paths.iter().map(String::as_str).collect::<HashSet<_>>();
        let mut matched = 0_usize;
        for path in paths {
            let actual = black_box(path.clone());
            assert!(expected.remove(actual.as_str()));
            matched += 1;
        }
        assert!(expected.is_empty());
        candidate = (matched, started.elapsed());
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
