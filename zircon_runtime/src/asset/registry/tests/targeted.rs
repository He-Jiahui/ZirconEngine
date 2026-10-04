use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use crate::asset::{AssetKind, AssetUri, AssetUuid};

use super::super::{AssetRegistryEntry, AssetRegistryIndex};
use super::{remove_dependency_paths, resolve_unique_dependencies};

const DEPENDENCY_PATHS: usize = 4_096;
const UNIQUE_DEPENDENCIES: usize = 256;
const BENCHMARK_ITERATIONS: usize = 16;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_id_runtime613_counted_removal_preserves_multiplicity_and_order() {
    let first = AssetUri::parse("res://dependency/first").unwrap();
    let second = AssetUri::parse("res://dependency/second").unwrap();
    let third = AssetUri::parse("res://dependency/third").unwrap();
    let missing = AssetUri::parse("res://dependency/missing").unwrap();
    let mut paths = vec![
        first.clone(),
        second.clone(),
        first.clone(),
        third.clone(),
        first.clone(),
        second.clone(),
    ];

    remove_dependency_paths(
        &mut paths,
        vec![first.clone(), second.clone(), first, missing],
    );

    assert_eq!(
        paths,
        vec![
            third,
            AssetUri::parse("res://dependency/first").unwrap(),
            second
        ]
    );
    let source = include_str!("../targeted.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    assert!(source.contains("HashMap::with_capacity(removed.len())"));
    assert!(source.contains("paths.retain"));
    assert!(!source.contains("paths.iter().position"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_id_runtime613_counted_dependency_path_removal_p95() {
    const PATHS: usize = 4_096;
    const REMOVED: usize = 2_048;
    const SAMPLE_PAIRS: usize = 17;
    let paths = (0..PATHS)
        .map(|index| {
            AssetUri::parse(&format!(
                "res://dependencies/long-segment-{index:05}/asset-{index:05}.zasset"
            ))
            .unwrap()
        })
        .collect::<Vec<_>>();
    let removed = paths
        .iter()
        .step_by(2)
        .take(REMOVED)
        .cloned()
        .collect::<Vec<_>>();
    let mut retired = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            retired.push(measure_path_removal(&paths, &removed, true));
            optimized.push(measure_path_removal(&paths, &removed, false));
        } else {
            optimized.push(measure_path_removal(&paths, &removed, false));
            retired.push(measure_path_removal(&paths, &removed, true));
        }
    }
    let retired_p95_ns = nearest_rank(&retired, 95);
    let optimized_p95_ns = nearest_rank(&optimized, 95);
    println!(
        "RUNTIME613_COUNTED_DEPENDENCY_PATH_REMOVAL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             paths={PATHS} removed={REMOVED} retired_p95_ns={retired_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} retired_raw_ns={} optimized_raw_ns={}",
        sample_csv(&retired),
        sample_csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(25),
        "counted removal P95 must be at most 25% of repeated position/remove: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_path_removal(paths: &[AssetUri], removed: &[AssetUri], retired: bool) -> u128 {
    let mut candidate = paths.to_vec();
    let started = Instant::now();
    if retired {
        legacy_remove_dependency_paths(&mut candidate, removed.iter().cloned());
    } else {
        remove_dependency_paths(&mut candidate, removed.to_vec());
    }
    black_box(candidate);
    started.elapsed().as_nanos().max(1)
}

fn legacy_remove_dependency_paths(
    paths: &mut Vec<AssetUri>,
    removed: impl IntoIterator<Item = AssetUri>,
) {
    for path in removed {
        if let Some(index) = paths.iter().position(|candidate| candidate == &path) {
            paths.remove(index);
        }
    }
}

#[test]
fn dependency_owner_refresh_deduplicates_in_first_path_order() {
    let (mut index, owner, mut paths, expected) = dependency_fixture(9, 3);
    let missing = AssetUri::parse("res://dependency/missing").unwrap();
    paths.push(missing.clone());
    index.dependency_paths_by_uuid.insert(owner, paths);

    index.refresh_dependency_owners(&HashSet::from([owner]));

    assert_eq!(index.get_dependencies_by_uuid(owner), expected);
    assert!(index.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        super::super::AssetRegistryDiagnostic::UnresolvedDependency {
            owner: diagnostic_owner,
            path,
        } if *diagnostic_owner == owner && path == &missing
    )));
}

#[test]
#[ignore = "release performance gate; run through the Runtime51 managed validator"]
fn asset_registry_dependency_owner_refresh_benchmark() {
    let (index, _owner, paths, expected) =
        dependency_fixture(DEPENDENCY_PATHS, UNIQUE_DEPENDENCIES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let measure_legacy = || {
            measure_ns(BENCHMARK_ITERATIONS, || {
                legacy_resolve_unique_dependencies(&paths, &index.uuids_by_path)
            })
        };
        let measure_optimized = || {
            measure_ns(BENCHMARK_ITERATIONS, || {
                resolve_unique_dependencies(&paths, &index.uuids_by_path).0
            })
        };
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    assert_eq!(
        resolve_unique_dependencies(&paths, &index.uuids_by_path).0,
        expected
    );
    println!(
        "PERF-MVP-556 task=asset_registry_dependency_owner_refresh sample_pairs={} dependency_paths={} unique_dependencies={} iterations={} legacy_samples_ns={} optimized_samples_ns={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={}",
        SAMPLE_PAIRS,
        DEPENDENCY_PATHS,
        UNIQUE_DEPENDENCIES,
        BENCHMARK_ITERATIONS,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
        "optimized P95 {optimized_p95}ns must be at most 75% of legacy P95 {legacy_p95}ns"
    );
}

fn dependency_fixture(
    path_count: usize,
    unique_count: usize,
) -> (AssetRegistryIndex, AssetUuid, Vec<AssetUri>, Vec<AssetUuid>) {
    assert!(unique_count > 0);
    let owner = AssetUuid::new();
    let dependencies = (0..unique_count)
        .map(|index| {
            let uuid = AssetUuid::new();
            let path = AssetUri::parse(&format!("res://dependency/{index}")).unwrap();
            (uuid, path)
        })
        .collect::<Vec<_>>();
    let entries = std::iter::once(AssetRegistryEntry::new(
        owner,
        AssetUri::parse("res://dependency/owner").unwrap(),
        AssetKind::Data,
        "owner",
    ))
    .chain(dependencies.iter().map(|(uuid, path)| {
        AssetRegistryEntry::new(*uuid, path.clone(), AssetKind::Data, "dependency")
    }));
    let index = AssetRegistryIndex::from_entries(entries).unwrap();
    let paths = (0..path_count)
        .map(|index| dependencies[index % unique_count].1.clone())
        .collect::<Vec<_>>();
    let expected = dependencies
        .iter()
        .map(|(uuid, _)| *uuid)
        .collect::<Vec<_>>();
    (index, owner, paths, expected)
}

fn legacy_resolve_unique_dependencies(
    paths: &[AssetUri],
    uuids_by_path: &std::collections::HashMap<AssetUri, AssetUuid>,
) -> Vec<AssetUuid> {
    let mut dependencies = Vec::new();
    for path in paths {
        if let Some(dependency) = uuids_by_path.get(path).copied() {
            if !dependencies.contains(&dependency) {
                dependencies.push(dependency);
            }
        }
    }
    dependencies
}

fn measure_ns(iterations: usize, mut resolve: impl FnMut() -> Vec<AssetUuid>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum ^= black_box(resolve()).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
