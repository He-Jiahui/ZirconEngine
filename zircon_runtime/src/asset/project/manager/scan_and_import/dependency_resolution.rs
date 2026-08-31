use std::collections::{HashMap, HashSet};

use crate::asset::{AssetId, AssetImportError, AssetUri, ImportedAsset};
use crate::core::resource::{
    ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceRegistryStaging,
};

use super::stage_project_resource;

#[derive(Default)]
struct ResolvedDependencies {
    dependency_ids: Vec<AssetId>,
    diagnostics: Vec<ResourceDiagnostic>,
}

fn resolve_dependencies(
    dependencies: &[AssetUri],
    registry: &ResourceRegistry,
) -> ResolvedDependencies {
    let mut resolved = ResolvedDependencies::default();
    let mut seen_dependency_ids = HashSet::with_capacity(dependencies.len());
    for dependency in dependencies {
        if let Some(record) = registry.get_by_locator(dependency) {
            admit_resolved_dependency_id(
                &mut resolved.dependency_ids,
                &mut seen_dependency_ids,
                record.id(),
            );
        } else {
            resolved.diagnostics.push(ResourceDiagnostic::error(format!(
                "unresolved asset dependency {dependency}"
            )));
        }
    }
    resolved
}

fn admit_resolved_dependency_id(
    dependency_ids: &mut Vec<AssetId>,
    seen_dependency_ids: &mut HashSet<AssetId>,
    dependency_id: AssetId,
) {
    if seen_dependency_ids.insert(dependency_id) {
        dependency_ids.push(dependency_id);
    }
}

pub(super) fn resolve_imported_dependencies(
    registry: &mut ResourceRegistryStaging,
    imported: &mut [ResourceRecord],
    dependencies_by_id: &HashMap<AssetId, Vec<AssetUri>>,
) -> Result<(), AssetImportError> {
    let resolved_by_id = dependencies_by_id
        .iter()
        .map(|(id, dependencies)| (*id, resolve_dependencies(dependencies, registry)))
        .collect::<HashMap<_, _>>();

    for record in imported.iter_mut() {
        apply_resolved_dependencies(record, &resolved_by_id);
        stage_project_resource(registry, record.clone())?;
    }
    Ok(())
}

fn apply_resolved_dependencies(
    record: &mut ResourceRecord,
    resolved_by_id: &HashMap<AssetId, ResolvedDependencies>,
) {
    let Some(resolved) = resolved_by_id.get(&record.id()) else {
        return;
    };
    record.dependency_ids = resolved.dependency_ids.clone();
    record
        .diagnostics
        .extend(resolved.diagnostics.iter().cloned());
}

pub(super) fn dependencies_for_entry(
    meta: &crate::asset::project::AssetMetaDocument,
    locator: &AssetUri,
) -> Vec<AssetUri> {
    meta.entries
        .iter()
        .find(|entry| &entry.url == locator)
        .map(|entry| entry.dependencies.clone())
        .unwrap_or_else(|| meta.dependencies.clone())
}

pub(super) fn merge_handwritten_dependencies_into_meta(
    meta: &mut crate::asset::project::AssetMetaDocument,
    asset: &ImportedAsset,
) {
    let dependencies =
        crate::asset::registry::dependency_extractors::handwritten_dependencies(asset);
    for dependency in dependencies {
        if !meta.dependencies.contains(&dependency) {
            meta.dependencies.push(dependency.clone());
        }
        if let Some(root) = meta
            .entries
            .iter_mut()
            .find(|entry| entry.url.label().is_none())
        {
            if !root.dependencies.contains(&dependency) {
                root.dependencies.push(dependency);
            }
        }
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::asset::AssetId;

    use super::admit_resolved_dependency_id;

    #[test]
    fn runtime85_project_dedup_recovery_batch_dependency_preserves_first_order() {
        let first = AssetId::new();
        let second = AssetId::new();
        let third = AssetId::new();
        let input = [second, first, second, third, first];
        let mut ordered = Vec::new();
        let mut seen = HashSet::new();

        for dependency_id in input {
            admit_resolved_dependency_id(&mut ordered, &mut seen, dependency_id);
        }

        assert_eq!(ordered, vec![second, first, third]);
    }

    #[test]
    fn runtime85_project_dedup_recovery_batch_dependency_uses_hash_admission() {
        const SOURCE: &str = include_str!("dependency_resolution.rs");
        let production = SOURCE.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("HashSet::with_capacity(dependencies.len())"));
        assert!(production.contains("admit_resolved_dependency_id("));
        assert!(!production.contains("resolved.dependency_ids.contains"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime85_project_dedup_recovery_batch_dependency_performance_evidence() {
        const DEPENDENCY_COUNT: usize = 4_096;
        const UNIQUE_DEPENDENCY_COUNT: usize = 1_024;
        const LEGACY_COMPARISONS: usize = 2_098_176;
        const SAMPLE_COUNT: usize = 21;
        let unique = (0..UNIQUE_DEPENDENCY_COUNT)
            .map(|_| AssetId::new())
            .collect::<Vec<_>>();
        let dependencies = (0..DEPENDENCY_COUNT)
            .map(|index| unique[index % UNIQUE_DEPENDENCY_COUNT])
            .collect::<Vec<_>>();

        let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
            || legacy_deduplicate(&dependencies),
            || hash_deduplicate(&dependencies),
        );
        assert_eq!(legacy_deduplicate(&dependencies), unique);
        assert_eq!(hash_deduplicate(&dependencies), unique);

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT RUNTIME85_PROJECT_DEPENDENCY_DEDUP_BENCH_V1 dependencies={DEPENDENCY_COUNT} unique_dependencies={UNIQUE_DEPENDENCY_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_linear_comparisons={LEGACY_COMPARISONS} optimized_hash_admissions={DEPENDENCY_COUNT} deterministic_admission_reduction_percent=99.8048 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    fn legacy_deduplicate(dependencies: &[AssetId]) -> Vec<AssetId> {
        let mut ordered = Vec::with_capacity(dependencies.len());
        for dependency_id in dependencies.iter().copied() {
            if !ordered.contains(&dependency_id) {
                ordered.push(dependency_id);
            }
        }
        ordered
    }

    fn hash_deduplicate(dependencies: &[AssetId]) -> Vec<AssetId> {
        let mut ordered = Vec::with_capacity(dependencies.len());
        let mut seen = HashSet::with_capacity(dependencies.len());
        for dependency_id in dependencies.iter().copied() {
            admit_resolved_dependency_id(&mut ordered, &mut seen, dependency_id);
        }
        ordered
    }

    fn benchmark_paired_samples<const SAMPLE_COUNT: usize>(
        mut legacy: impl FnMut() -> Vec<AssetId>,
        mut optimized: impl FnMut() -> Vec<AssetId>,
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

    fn benchmark_sample(operation: &mut impl FnMut() -> Vec<AssetId>) -> u128 {
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
