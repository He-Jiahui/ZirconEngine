use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

const MENU_PATH_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn menu_paths() -> Vec<String> {
    (0..MENU_PATH_COUNT)
        .map(|index| {
            format!("asset/document/generated/validation/category/action/command.{index:05}")
        })
        .collect()
}

fn ordered_unique_count(paths: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    paths
        .iter()
        .filter(|path| unique.insert(path.as_str()))
        .count()
}

fn hash_unique_count(paths: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(paths.len());
    paths
        .iter()
        .filter(|path| unique.insert(path.as_str()))
        .count()
}

#[test]
fn optimization_batch_ih_editor618_menu_path_validation_uses_preallocated_hash_membership() {
    let source = include_str!("../../registry.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(production
        .contains("let mut paths = HashSet::with_capacity(descriptor.menu_items().len());"));
    assert!(production.contains("paths.insert(menu_item.path())"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_ih_editor618_hash_menu_path_validation_performance_evidence() {
    let paths = menu_paths();
    assert_eq!(ordered_unique_count(&paths), hash_unique_count(&paths));

    black_box(ordered_unique_count(black_box(&paths)));
    black_box(hash_unique_count(black_box(&paths)));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&paths)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&paths)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&paths)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&paths)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR618_HASH_MENU_PATH_VALIDATION_BENCH_V1 \
         paths={MENU_PATH_COUNT} borrowed_identity=true \
         ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 40,
        "hash menu-path validation P95 {:?} exceeded 40% of ordered P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
