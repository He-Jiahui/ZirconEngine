//! Admission checks that must finish before an editor-plugin catalog is published.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::catalog::EditorPluginCatalog;

/// A structural error that prevents a catalog generation from becoming visible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorPluginCatalogAdmissionError {
    DuplicatePackage { package_id: String },
    DependencyCycle { package_ids: Vec<String> },
}

impl fmt::Display for EditorPluginCatalogAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePackage { package_id } => {
                write!(
                    formatter,
                    "editor plugin catalog contains duplicate package `{package_id}`"
                )
            }
            Self::DependencyCycle { package_ids } => write!(
                formatter,
                "editor plugin catalog contains dependency cycle {}",
                package_ids.join(" -> ")
            ),
        }
    }
}

impl std::error::Error for EditorPluginCatalogAdmissionError {}

/// Rejects a catalog whose declared package dependencies form a cycle.
///
/// Native ABI and engine-version checks stay with the runtime native loader. This editor-side
/// admission boundary only checks the in-memory package graph before a generation is published.
pub(super) fn validate_catalog_admission(
    catalog: &EditorPluginCatalog,
) -> Result<(), EditorPluginCatalogAdmissionError> {
    if let Some(package_id) = catalog.admission_duplicate_package_ids().iter().next() {
        return Err(EditorPluginCatalogAdmissionError::DuplicatePackage {
            package_id: package_id.clone(),
        });
    }
    let mut dependencies_by_package = BTreeMap::<String, BTreeSet<String>>::new();
    for package in catalog.package_manifests() {
        let dependencies = package
            .dependencies
            .into_iter()
            .map(|dependency| dependency.id)
            .collect();
        if dependencies_by_package
            .insert(package.id.clone(), dependencies)
            .is_some()
        {
            return Err(EditorPluginCatalogAdmissionError::DuplicatePackage {
                package_id: package.id,
            });
        }
    }

    if let Some(package_ids) = find_dependency_cycle(&dependencies_by_package) {
        return Err(EditorPluginCatalogAdmissionError::DependencyCycle { package_ids });
    }
    Ok(())
}

fn find_dependency_cycle(
    dependencies_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> Option<Vec<String>> {
    let mut completed = BTreeSet::<&str>::new();
    let mut visiting = BTreeSet::<&str>::new();
    let mut path = Vec::<&str>::new();
    for package_id in dependencies_by_package.keys() {
        if let Some(cycle) = visit_dependency(
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

fn visit_dependency<'a>(
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
                if let Some(cycle) = visit_dependency(
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::plugin::{PluginDependencyManifest, PluginPackageManifest};

    use crate::core::plugin::{EditorPluginCatalog, EditorPluginDescriptor};

    use super::{
        find_dependency_cycle, validate_catalog_admission, EditorPluginCatalogAdmissionError,
    };

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
        const SOURCE: &str = include_str!("admission.rs");
        let production = SOURCE.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("BTreeSet::<&str>::new()"));
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
            .map(|(package_id, _)| {
                EditorPluginDescriptor::new(*package_id, *package_id, *package_id)
            })
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
}
