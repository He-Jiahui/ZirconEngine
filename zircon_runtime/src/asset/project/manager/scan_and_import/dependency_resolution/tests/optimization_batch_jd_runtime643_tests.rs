use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const EXISTING_DEPENDENCY_COUNT: usize = 4_096;
const INCOMING_DEPENDENCY_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 17;
const BORROWED_BENCH_COUNT: usize = 4_096;
const BORROWED_SAMPLE_PAIRS: usize = 101;

#[test]
fn optimization_batch_jd_runtime643_indexes_handwritten_dependency_merges() {
    let source = include_str!("../../dependency_resolution.rs");
    let merge = source
        .split("pub(super) fn merge_handwritten_dependencies_into_meta")
        .nth(1)
        .expect("handwritten dependency merge remains present")
        .split("#[cfg(test)]")
        .next()
        .expect("handwritten dependency merge remains bounded");

    assert!(merge.contains("HashSet::with_capacity(meta_dependency_capacity)"));
    assert!(merge.contains("HashSet::with_capacity(root_dependency_capacity)"));
    assert!(merge.contains("if dependencies.is_empty()"));
    assert_eq!(
        merge
            .matches(".find(|entry| entry.url.label().is_none())")
            .count(),
        1
    );
    assert!(!merge.contains("meta.dependencies.contains(&dependency)"));
    assert!(!merge.contains("root.dependencies.contains(&dependency)"));
}

#[test]
fn optimization_batch_20260921_runtime865_borrowed_dependency_indexes_preserve_dual_destination_order(
) {
    let meta = vec![uri("meta/a"), uri("shared/b")];
    let root = vec![uri("shared/b"), uri("root/c")];
    let incoming = vec![
        uri("shared/b"),
        uri("new/d"),
        uri("meta/a"),
        uri("new/e"),
        uri("new/d"),
    ];

    let expected = owned_index_merge_uris(meta.clone(), root.clone(), incoming.clone());
    let actual = borrowed_index_merge_uris(meta, root, incoming);
    assert_eq!(actual, expected);
    assert_eq!(
        actual.0,
        vec![uri("meta/a"), uri("shared/b"), uri("new/d"), uri("new/e")]
    );
    assert_eq!(
        actual.1,
        vec![
            uri("shared/b"),
            uri("root/c"),
            uri("new/d"),
            uri("meta/a"),
            uri("new/e"),
        ]
    );

    let source = include_str!("../../dependency_resolution.rs");
    let merge = source
        .split("pub(super) fn merge_handwritten_dependencies_into_meta")
        .nth(1)
        .expect("handwritten dependency merge remains present")
        .split("#[cfg(test)]")
        .next()
        .expect("handwritten dependency merge remains bounded");
    assert!(merge.contains("HashSet<&AssetUri>"));
    assert!(merge.contains("admission_flags"));
    assert!(!merge.contains(".iter().cloned()"));
    assert_eq!(merge.matches("dependency.clone()").count(), 1);
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jd_runtime643_indexed_handwritten_dependency_merge_benchmark() {
    let existing = (0..EXISTING_DEPENDENCY_COUNT).collect::<Vec<_>>();
    let incoming = (EXISTING_DEPENDENCY_COUNT / 2
        ..EXISTING_DEPENDENCY_COUNT / 2 + INCOMING_DEPENDENCY_COUNT)
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_merge(&existing, &incoming),
        indexed_merge(&existing, &incoming)
    );

    for _ in 0..4 {
        black_box(measure_merge(&existing, &incoming, false));
        black_box(measure_merge(&existing, &incoming, true));
    }

    let mut linear_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            linear_samples.push(measure_merge(&existing, &incoming, false));
            indexed_samples.push(measure_merge(&existing, &incoming, true));
        } else {
            indexed_samples.push(measure_merge(&existing, &incoming, true));
            linear_samples.push(measure_merge(&existing, &incoming, false));
        }
    }

    let linear_p95 = percentile(&linear_samples, 95);
    let indexed_p95 = percentile(&indexed_samples, 95);
    let improvement_percent =
        linear_p95.saturating_sub(indexed_p95).saturating_mul(100) / linear_p95.max(1);
    println!(
        "RUNTIME643_INDEXED_HANDWRITTEN_DEPENDENCY_MERGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} existing_dependencies={EXISTING_DEPENDENCY_COUNT} incoming_dependencies={INCOMING_DEPENDENCY_COUNT} linear_ns={} indexed_ns={} linear_p95_ns={linear_p95} indexed_p95_ns={indexed_p95} improvement_percent={improvement_percent} threshold_percent=50",
        csv(&linear_samples),
        csv(&indexed_samples),
    );
    assert!(indexed_p95 <= linear_p95 * 50 / 100);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260921_runtime865_handwritten_meta_dependency_borrowed_index_bench() {
    let meta = uri_range("meta/existing/long/catalog/path", BORROWED_BENCH_COUNT);
    let root = uri_range("root/existing/long/catalog/path", BORROWED_BENCH_COUNT);
    let incoming = uri_range("incoming/distinct/long/catalog/path", BORROWED_BENCH_COUNT);
    assert_eq!(
        owned_index_merge_uris(meta.clone(), root.clone(), incoming.clone()),
        borrowed_index_merge_uris(meta.clone(), root.clone(), incoming.clone())
    );

    let mut owned_index_samples = Vec::with_capacity(BORROWED_SAMPLE_PAIRS);
    let mut borrowed_index_samples = Vec::with_capacity(BORROWED_SAMPLE_PAIRS);
    for pair in 0..BORROWED_SAMPLE_PAIRS {
        let owned_inputs = (meta.clone(), root.clone(), incoming.clone());
        let borrowed_inputs = (meta.clone(), root.clone(), incoming.clone());
        if pair % 2 == 0 {
            owned_index_samples.push(measure_uri_merge(|| {
                owned_index_merge_uris(owned_inputs.0, owned_inputs.1, owned_inputs.2)
            }));
            borrowed_index_samples.push(measure_uri_merge(|| {
                borrowed_index_merge_uris(borrowed_inputs.0, borrowed_inputs.1, borrowed_inputs.2)
            }));
        } else {
            borrowed_index_samples.push(measure_uri_merge(|| {
                borrowed_index_merge_uris(borrowed_inputs.0, borrowed_inputs.1, borrowed_inputs.2)
            }));
            owned_index_samples.push(measure_uri_merge(|| {
                owned_index_merge_uris(owned_inputs.0, owned_inputs.1, owned_inputs.2)
            }));
        }
    }

    let owned_index_p50 = percentile(&owned_index_samples, 50);
    let owned_index_p95 = percentile(&owned_index_samples, 95);
    let owned_index_p99 = percentile(&owned_index_samples, 99);
    let borrowed_index_p50 = percentile(&borrowed_index_samples, 50);
    let borrowed_index_p95 = percentile(&borrowed_index_samples, 95);
    let borrowed_index_p99 = percentile(&borrowed_index_samples, 99);
    println!(
        "RUNTIME865_HANDWRITTEN_META_DEPENDENCY_BORROWED_INDEX_BENCH_V1 sample_pairs={BORROWED_SAMPLE_PAIRS} existing_per_destination={BORROWED_BENCH_COUNT} incoming_dependencies={BORROWED_BENCH_COUNT} owned_index_p50_ns={owned_index_p50} owned_index_p95_ns={owned_index_p95} owned_index_p99_ns={owned_index_p99} borrowed_index_p50_ns={borrowed_index_p50} borrowed_index_p95_ns={borrowed_index_p95} borrowed_index_p99_ns={borrowed_index_p99} modeled_uri_clones_before={} modeled_uri_clones_after={}",
        BORROWED_BENCH_COUNT * 5,
        BORROWED_BENCH_COUNT,
    );
    assert!(
        borrowed_index_p95 < owned_index_p95,
        "expected borrowed indexes to improve P95, got owned={owned_index_p95}ns borrowed={borrowed_index_p95}ns"
    );
}

fn measure_merge(existing: &[usize], incoming: &[usize], indexed: bool) -> u128 {
    let started = Instant::now();
    let merged = if indexed {
        indexed_merge(existing, incoming)
    } else {
        legacy_merge(existing, incoming)
    };
    black_box(merged);
    started.elapsed().as_nanos().max(1)
}

fn legacy_merge(existing: &[usize], incoming: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let mut meta_dependencies = existing.to_vec();
    let mut root_dependencies = existing.to_vec();
    for dependency in incoming.iter().copied() {
        if !meta_dependencies.contains(&dependency) {
            meta_dependencies.push(dependency);
        }
        if !root_dependencies.contains(&dependency) {
            root_dependencies.push(dependency);
        }
    }
    (meta_dependencies, root_dependencies)
}

fn indexed_merge(existing: &[usize], incoming: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let capacity = existing.len().saturating_add(incoming.len());
    let mut meta_dependencies = existing.to_vec();
    let mut root_dependencies = existing.to_vec();
    let mut meta_index = HashSet::with_capacity(capacity);
    let mut root_index = HashSet::with_capacity(capacity);
    meta_index.extend(existing.iter().copied());
    root_index.extend(existing.iter().copied());
    for dependency in incoming.iter().copied() {
        if meta_index.insert(dependency) {
            meta_dependencies.push(dependency);
        }
        if root_index.insert(dependency) {
            root_dependencies.push(dependency);
        }
    }
    (meta_dependencies, root_dependencies)
}

fn owned_index_merge_uris(
    mut meta: Vec<String>,
    mut root: Vec<String>,
    incoming: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    let capacity = meta.len().saturating_add(incoming.len());
    let mut meta_index = HashSet::with_capacity(capacity);
    let mut root_index = HashSet::with_capacity(capacity);
    meta_index.extend(meta.iter().cloned());
    root_index.extend(root.iter().cloned());
    for dependency in incoming {
        if meta_index.insert(dependency.clone()) {
            meta.push(dependency.clone());
        }
        if root_index.insert(dependency.clone()) {
            root.push(dependency);
        }
    }
    (meta, root)
}

fn borrowed_index_merge_uris(
    mut meta: Vec<String>,
    mut root: Vec<String>,
    incoming: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    const META_ADMISSION: u8 = 1;
    const ROOT_ADMISSION: u8 = 1 << 1;
    let capacity = meta.len().saturating_add(incoming.len());
    let mut meta_index = HashSet::with_capacity(capacity);
    let mut root_index = HashSet::with_capacity(capacity);
    meta_index.extend(meta.iter());
    root_index.extend(root.iter());
    let mut admission_flags = Vec::with_capacity(incoming.len());
    let mut meta_additions = 0;
    let mut root_additions = 0;
    for dependency in &incoming {
        let meta_is_new = meta_index.insert(dependency);
        let root_is_new = root_index.insert(dependency);
        admission_flags
            .push(u8::from(meta_is_new) * META_ADMISSION + u8::from(root_is_new) * ROOT_ADMISSION);
        meta_additions += usize::from(meta_is_new);
        root_additions += usize::from(root_is_new);
    }
    drop(meta_index);
    drop(root_index);
    meta.reserve(meta_additions);
    root.reserve(root_additions);
    for (dependency, flags) in incoming.into_iter().zip(admission_flags) {
        match (flags & META_ADMISSION != 0, flags & ROOT_ADMISSION != 0) {
            (true, true) => {
                meta.push(dependency.clone());
                root.push(dependency);
            }
            (true, false) => meta.push(dependency),
            (false, true) => root.push(dependency),
            (false, false) => {}
        }
    }
    (meta, root)
}

fn uri_range(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| uri(&format!("{prefix}/{index:05}.asset")))
        .collect()
}

fn uri(value: &str) -> String {
    format!("res://{value}")
}

fn measure_uri_merge(work: impl FnOnce() -> (Vec<String>, Vec<String>)) -> u128 {
    let started = Instant::now();
    black_box(work());
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
