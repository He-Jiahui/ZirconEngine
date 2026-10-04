use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::plugin::{PluginDependencyManifest, PluginPackageManifest};

use crate::core::plugin::{EditorPluginCatalog, EditorPluginDescriptor};

use super::{find_dependency_cycle, validate_catalog_admission, EditorPluginCatalogAdmissionError};

#[test]
fn rejects_a_cycle_between_declared_catalog_packages() {
    let catalog = catalog_with_dependencies(&[
        ("plugin.alpha", "plugin.beta"),
        ("plugin.beta", "plugin.alpha"),
    ]);

    assert_eq!(
        validate_catalog_admission(&catalog),
        Err(EditorPluginCatalogAdmissionError::DependencyCycle {
            package_ids: vec![
                "plugin.alpha".to_string(),
                "plugin.beta".to_string(),
                "plugin.alpha".to_string(),
            ],
        })
    );
}

#[test]
fn ignores_dependencies_outside_the_published_catalog() {
    let catalog = catalog_with_dependencies(&[("plugin.alpha", "plugin.external")]);

    assert_eq!(validate_catalog_admission(&catalog), Ok(()));
}

#[test]
fn rejects_duplicate_runtime_manifest_input_for_one_editor_package() {
    let catalog = EditorPluginCatalog::from_descriptors(
        [EditorPluginDescriptor::new(
            "plugin.alpha",
            "Alpha",
            "alpha",
        )],
        [
            PluginPackageManifest::new("plugin.alpha", "Alpha"),
            PluginPackageManifest::new("plugin.alpha", "Conflicting Alpha"),
        ],
    );

    assert_eq!(
        validate_catalog_admission(&catalog),
        Err(EditorPluginCatalogAdmissionError::DuplicatePackage {
            package_id: "plugin.alpha".to_string(),
        })
    );
}

#[test]
fn ignores_duplicate_runtime_only_manifest_input() {
    let catalog = EditorPluginCatalog::from_descriptors(
        [EditorPluginDescriptor::new(
            "plugin.alpha",
            "Alpha",
            "alpha",
        )],
        [
            PluginPackageManifest::new("plugin.alpha", "Alpha"),
            PluginPackageManifest::new("runtime.only", "Runtime Only"),
            PluginPackageManifest::new("runtime.only", "Conflicting Runtime Only"),
        ],
    );

    assert_eq!(validate_catalog_admission(&catalog), Ok(()));
}

#[test]
fn optimization_wave_20260824i_editor06_plugin_admission_borrowed_dfs_preserves_cycle_path() {
    let dependencies = BTreeMap::from([
        (
            "plugin.alpha".to_string(),
            BTreeSet::from(["plugin.beta".to_string()]),
        ),
        (
            "plugin.beta".to_string(),
            BTreeSet::from(["plugin.gamma".to_string()]),
        ),
        (
            "plugin.gamma".to_string(),
            BTreeSet::from(["plugin.beta".to_string()]),
        ),
    ]);

    assert_eq!(
        find_dependency_cycle(&dependencies),
        Some(vec![
            "plugin.beta".to_string(),
            "plugin.gamma".to_string(),
            "plugin.beta".to_string(),
        ])
    );
}

#[test]
fn optimization_wave_20260824i_editor06_plugin_admission_borrowed_dfs_uses_borrowed_ids() {
    const SOURCE: &str = include_str!("../admission.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("HashSet::with_capacity(dependencies_by_package.len())"));
    assert!(production.contains("Vec::<&str>::new()"));
    assert!(production.contains("completed.insert(package_id)"));
    assert!(!production.contains("visiting.insert(package_id.to_string())"));
    assert!(!production.contains("path.push(package_id.to_string())"));
    assert!(!production.contains("completed.insert(package_id.to_string())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_wave_20260824i_editor06_plugin_admission_borrowed_dfs_evidence() {
    const PACKAGE_COUNT: usize = 4_096;
    const PACKAGE_ID_BYTES: usize = 512;
    const LEGACY_PACKAGE_ID_CLONES: usize = PACKAGE_COUNT * 3;
    const SAMPLE_COUNT: usize = 21;
    let suffix = "x".repeat(PACKAGE_ID_BYTES - 9);
    let dependencies = (0..PACKAGE_COUNT)
        .map(|index| (format!("{index:08}-{suffix}"), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_find_dependency_cycle(black_box(&dependencies)),
        || find_dependency_cycle(black_box(&dependencies)),
    );
    assert_eq!(legacy_find_dependency_cycle(&dependencies), None);
    assert_eq!(find_dependency_cycle(&dependencies), None);

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR06_PLUGIN_ADMISSION_BORROWED_DFS_BENCH_V1 packages={PACKAGE_COUNT} package_id_bytes={PACKAGE_ID_BYTES} samples={SAMPLE_COUNT} sample_order=alternating legacy_package_id_clones={LEGACY_PACKAGE_ID_CLONES} optimized_package_id_clones=0 deterministic_package_id_clone_reduction_percent=100.0000 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
    );
}

fn catalog_with_dependencies(dependencies: &[(&str, &str)]) -> EditorPluginCatalog {
    let descriptors = dependencies
        .iter()
        .map(|(package_id, _)| EditorPluginDescriptor::new(*package_id, *package_id, *package_id))
        .collect::<Vec<_>>();
    let manifests = dependencies
        .iter()
        .map(|(package_id, dependency_id)| {
            let mut manifest = PluginPackageManifest::new(*package_id, *package_id);
            manifest
                .dependencies
                .push(PluginDependencyManifest::new(*dependency_id, true));
            manifest
        })
        .collect::<Vec<_>>();
    EditorPluginCatalog::from_descriptors(descriptors, manifests)
}

fn legacy_find_dependency_cycle(
    dependencies_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> Option<Vec<String>> {
    let mut completed = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    let mut path = Vec::new();
    for package_id in dependencies_by_package.keys() {
        if let Some(cycle) = legacy_visit_dependency(
            package_id,
            dependencies_by_package,
            &mut completed,
            &mut visiting,
            &mut path,
        ) {
            return Some(cycle);
        }
    }
    None
}

fn ordered_borrowed_find_dependency_cycle(
    dependencies_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> Option<Vec<String>> {
    let mut completed = BTreeSet::<&str>::new();
    let mut visiting = BTreeSet::<&str>::new();
    let mut path = Vec::<&str>::new();
    for package_id in dependencies_by_package.keys() {
        if let Some(cycle) = ordered_borrowed_visit_dependency(
            package_id.as_str(),
            dependencies_by_package,
            &mut completed,
            &mut visiting,
            &mut path,
        ) {
            return Some(cycle);
        }
    }
    None
}

fn ordered_borrowed_visit_dependency<'a>(
    package_id: &'a str,
    dependencies_by_package: &'a BTreeMap<String, BTreeSet<String>>,
    completed: &mut BTreeSet<&'a str>,
    visiting: &mut BTreeSet<&'a str>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    if completed.contains(package_id) {
        return None;
    }
    if !visiting.insert(package_id) {
        let cycle_start = path
            .iter()
            .position(|candidate| *candidate == package_id)
            .expect("a visiting package is always on the dependency path");
        let mut cycle = path[cycle_start..]
            .iter()
            .map(|package_id| (*package_id).to_string())
            .collect::<Vec<_>>();
        cycle.push(package_id.to_string());
        return Some(cycle);
    }
    path.push(package_id);
    if let Some(dependencies) = dependencies_by_package.get(package_id) {
        for dependency_id in dependencies {
            if dependencies_by_package.contains_key(dependency_id) {
                if let Some(cycle) = ordered_borrowed_visit_dependency(
                    dependency_id.as_str(),
                    dependencies_by_package,
                    completed,
                    visiting,
                    path,
                ) {
                    return Some(cycle);
                }
            }
        }
    }
    path.pop();
    visiting.remove(package_id);
    completed.insert(package_id);
    None
}

#[test]
fn optimization_batch_ig_editor617_hash_dfs_preserves_deterministic_cycle_path() {
    let dependencies = BTreeMap::from([
        (
            "plugin.alpha".to_string(),
            BTreeSet::from(["plugin.beta".to_string()]),
        ),
        (
            "plugin.beta".to_string(),
            BTreeSet::from(["plugin.gamma".to_string()]),
        ),
        (
            "plugin.gamma".to_string(),
            BTreeSet::from(["plugin.beta".to_string()]),
        ),
    ]);

    assert_eq!(
        find_dependency_cycle(&dependencies),
        ordered_borrowed_find_dependency_cycle(&dependencies)
    );
}

#[test]
fn optimization_batch_ig_editor617_dfs_uses_preallocated_hash_membership() {
    let source = include_str!("../admission.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, BTreeSet, HashSet};"));
    assert_eq!(
        production
            .matches("HashSet::with_capacity(dependencies_by_package.len())")
            .count(),
        2
    );
    assert!(production.contains("completed: &mut HashSet<&'a str>"));
    assert!(production.contains("visiting: &mut HashSet<&'a str>"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_ig_editor617_hash_plugin_admission_dfs_performance_evidence() {
    const PACKAGES: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let suffix = "x".repeat(192);
    let dependencies = (0..PACKAGES)
        .map(|index| {
            (
                format!("plugin.admission.{suffix}.{index:05}"),
                BTreeSet::new(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(ordered_borrowed_find_dependency_cycle(&dependencies), None);
    assert_eq!(find_dependency_cycle(&dependencies), None);
    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_borrowed_find_dependency_cycle(black_box(
                &dependencies,
            )));
            ordered_samples.push(started.elapsed().as_nanos().max(1));
            let started = Instant::now();
            black_box(find_dependency_cycle(black_box(&dependencies)));
            hash_samples.push(started.elapsed().as_nanos().max(1));
        } else {
            let started = Instant::now();
            black_box(find_dependency_cycle(black_box(&dependencies)));
            hash_samples.push(started.elapsed().as_nanos().max(1));
            let started = Instant::now();
            black_box(ordered_borrowed_find_dependency_cycle(black_box(
                &dependencies,
            )));
            ordered_samples.push(started.elapsed().as_nanos().max(1));
        }
    }
    ordered_samples.sort_unstable();
    hash_samples.sort_unstable();
    let ordered_p95 = ordered_samples[(SAMPLE_PAIRS - 1) * 95 / 100];
    let hash_p95 = hash_samples[(SAMPLE_PAIRS - 1) * 95 / 100];
    println!(
        "EDITOR617_HASH_PLUGIN_ADMISSION_DFS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} packages={PACKAGES} package_id_bytes={} ordered_p95_ns={ordered_p95} hash_p95_ns={hash_p95} target_ratio_bp=4000",
        dependencies.keys().next().unwrap().len(),
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= ordered_p95.saturating_mul(4_000),
        "hash DFS P95 {hash_p95} ns exceeded 40% of ordered P95 {ordered_p95} ns"
    );
}

fn legacy_visit_dependency(
    package_id: &str,
    dependencies_by_package: &BTreeMap<String, BTreeSet<String>>,
    completed: &mut BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if completed.contains(package_id) {
        return None;
    }
    if !visiting.insert(package_id.to_string()) {
        let cycle_start = path
            .iter()
            .position(|candidate| candidate == package_id)
            .expect("a visiting package is always on the dependency path");
        let mut cycle = path[cycle_start..].to_vec();
        cycle.push(package_id.to_string());
        return Some(cycle);
    }

    path.push(package_id.to_string());
    if let Some(dependencies) = dependencies_by_package.get(package_id) {
        for dependency_id in dependencies {
            if dependencies_by_package.contains_key(dependency_id) {
                if let Some(cycle) = legacy_visit_dependency(
                    dependency_id,
                    dependencies_by_package,
                    completed,
                    visiting,
                    path,
                ) {
                    return Some(cycle);
                }
            }
        }
    }
    path.pop();
    visiting.remove(package_id);
    completed.insert(package_id.to_string());
    None
}

fn benchmark_paired_samples<const SAMPLE_COUNT: usize>(
    mut legacy: impl FnMut() -> Option<Vec<String>>,
    mut optimized: impl FnMut() -> Option<Vec<String>>,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> Option<Vec<String>>) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}
