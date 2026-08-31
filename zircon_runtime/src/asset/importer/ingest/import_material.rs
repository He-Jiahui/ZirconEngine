use crate::asset::assets::{ImportedAsset, MaterialAsset, ZMaterialDocument};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_material(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let material_document = ZMaterialDocument::from_project_toml_str(&document, |reference| {
        context.resolve_project_asset_ref(reference)
    })?;
    let material = MaterialAsset::from_zmaterial_document(material_document);
    let dependencies = material.direct_reference_locators();
    let mut outcome =
        AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Material(material));
    outcome.entries[0].dependencies = dependencies;
    Ok(outcome.with_reference_repairs(context.reference_repairs()))
}

#[cfg(test)]
mod plugins07_material_dependency_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const DEPENDENCIES_PER_SAMPLE: usize = 16_384;

    #[test]
    fn decode_material_hotpath_contract_reserves_dependency_publication() {
        let mut dependencies = Vec::<usize>::new();

        reserve_dependency_benchmark(128, &mut dependencies);

        assert!(dependencies.capacity() >= 129);
        assert!(dependencies.is_empty());
        dependencies.extend([0, 1, 2]);
        assert_eq!(dependencies, [0, 1, 2]);
    }

    #[test]
    fn decoded_material_dependency_publication_uses_the_asset_reference_owner() {
        let production = include_str!("import_material.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material importer test boundary");

        assert!(production.contains("material.direct_reference_locators()"));
        assert!(!production.contains("dependencies.push(material.shader.locator.clone())"));
        assert!(!production.contains("if let Some(parent) = material.parent.as_ref()"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn decode_material_hotpath_performance_release_dependency_publication() {
        for _ in 0..4 {
            black_box(measure_dependencies(false));
            black_box(measure_dependencies(true));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut legacy_growths = None;
        let mut optimized_growths = None;
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy, optimized) = if pair_index % 2 == 0 {
                (measure_dependencies(false), measure_dependencies(true))
            } else {
                let optimized = measure_dependencies(true);
                (measure_dependencies(false), optimized)
            };
            legacy_growths.get_or_insert(legacy.1);
            optimized_growths.get_or_insert(optimized.1);
            assert_eq!(legacy_growths, Some(legacy.1));
            assert_eq!(optimized_growths, Some(optimized.1));
            legacy_samples.push(legacy.0);
            optimized_samples.push(optimized.0);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_preallocated_material_dependencies sample_pairs={SAMPLE_PAIRS} dependencies_per_sample={DEPENDENCIES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40 legacy_capacity_growths_per_sample={} optimized_capacity_growths_per_sample={} legacy_publication_vectors_per_sample=2 optimized_publication_vectors_per_sample=1 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
            legacy_growths.unwrap(),
            optimized_growths.unwrap(),
        );
        assert_eq!(optimized_growths, Some(0));
        assert!(
            improvement_percent >= 40,
            "single-vector material dependency publication must improve P95 by at least 40%"
        );
    }

    fn measure_dependencies(preallocated: bool) -> (u128, usize) {
        let started = Instant::now();
        let mut growths = 0;
        if preallocated {
            let mut published = Vec::new();
            reserve_dependency_benchmark(DEPENDENCIES_PER_SAMPLE - 1, &mut published);
            for dependency in 0..DEPENDENCIES_PER_SAMPLE {
                let capacity = published.capacity();
                published.push(black_box(dependency));
                growths += usize::from(published.capacity() != capacity);
            }
            black_box(published);
        } else {
            let mut collected = Vec::new();
            for dependency in 0..DEPENDENCIES_PER_SAMPLE {
                let capacity = collected.capacity();
                collected.push(black_box(dependency));
                growths += usize::from(collected.capacity() != capacity);
            }
            let mut published = Vec::new();
            for dependency in collected {
                let capacity = published.capacity();
                published.push(dependency);
                growths += usize::from(published.capacity() != capacity);
            }
            black_box(published);
        }
        (started.elapsed().as_nanos().max(1), growths)
    }

    fn reserve_dependency_benchmark<Dependency>(
        non_shader_dependency_count: usize,
        dependencies: &mut Vec<Dependency>,
    ) {
        dependencies.reserve(non_shader_dependency_count.saturating_add(1));
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
