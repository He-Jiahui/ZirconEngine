use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::asset::AssetUuid;

#[test]
fn restored_non_shader_records_are_filtered_before_artifact_reads() {
    let paths = ProjectPaths::from_root(std::env::temp_dir()).unwrap();
    let missing_artifact = AssetUri::parse("lib://data/missing.zasset").unwrap();
    let record = ResourceRecord::new(
        AssetId::from_asset_uuid(AssetUuid::new()),
        AssetKind::Data,
        AssetUri::parse("res://data/missing.json").unwrap(),
    )
    .with_artifact_locator(missing_artifact);

    let index =
        ShaderImportDependencyIndex::from_artifacts(&ArtifactStore::default(), &paths, &[record])
            .expect("non-shader records must not enter dependency-index artifact reads");

    assert!(index.shaders_by_id.is_empty());
}

#[test]
fn optimization_wave_20260824i_runtime91_shader_import_provider_dedup_preserves_first_order() {
    let consumer_id = AssetId::new();
    let first_provider_id = AssetId::new();
    let second_provider_id = AssetId::new();
    let first_locator = AssetUri::parse("res://shaders/includes/first.zshader").unwrap();
    let second_locator = AssetUri::parse("res://shaders/includes/second.zshader").unwrap();
    let mut index = ShaderImportDependencyIndex::default();
    index.shaders_by_id.insert(
        first_provider_id,
        indexed_shader(first_locator.clone(), Some("include.first"), &[]),
    );
    index.shaders_by_id.insert(
        second_provider_id,
        indexed_shader(second_locator.clone(), Some("include.second"), &[]),
    );
    index.shaders_by_id.insert(
        consumer_id,
        indexed_shader(
            AssetUri::parse("res://shaders/consumer.zshader").unwrap(),
            None,
            &["include.second", "include.first", "include.second"],
        ),
    );
    index.includes_by_path.insert(
        "include.first".to_string(),
        HashSet::from([first_provider_id]),
    );
    index.includes_by_path.insert(
        "include.second".to_string(),
        HashSet::from([second_provider_id]),
    );

    assert_eq!(
        index.dependency_locators(consumer_id),
        vec![second_locator, first_locator]
    );
}

#[test]
fn optimization_wave_20260824i_runtime91_shader_import_provider_dedup_uses_id_admission() {
    const SOURCE: &str = include_str!("../shader_import_dependencies.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("seen_provider_ids.insert(provider_id)"));
    assert!(production.contains("provider.locator.clone()"));
    assert!(!production.contains("dependencies.contains(&locator)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_wave_20260824i_runtime91_shader_import_provider_dedup_evidence() {
    const IMPORT_COUNT: usize = 4_096;
    const PROVIDER_COUNT: usize = 1_024;
    const LEGACY_COMPARISONS: usize = 2_098_176;
    const SAMPLE_COUNT: usize = 21;
    let providers = (0..PROVIDER_COUNT)
        .map(|index| {
            (
                AssetId::new(),
                AssetUri::parse(&format!(
                    "res://shaders/includes/provider-{index:04}-long-module-name.zshader"
                ))
                .unwrap(),
            )
        })
        .collect::<HashMap<_, _>>();
    let provider_ids = providers.keys().copied().collect::<Vec<_>>();
    let imports = (0..IMPORT_COUNT)
        .map(|index| provider_ids[index % PROVIDER_COUNT])
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_provider_locators(&imports, &providers),
        || hashed_provider_locators(&imports, &providers),
    );
    assert_eq!(
        legacy_provider_locators(&imports, &providers),
        hashed_provider_locators(&imports, &providers)
    );

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME91_SHADER_IMPORT_PROVIDER_DEDUP_BENCH_V1 imports={IMPORT_COUNT} unique_providers={PROVIDER_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_linear_uri_comparisons={LEGACY_COMPARISONS} optimized_id_hash_admissions={IMPORT_COUNT} deterministic_admission_reduction_percent=99.8048 legacy_uri_clones={IMPORT_COUNT} optimized_uri_clones={PROVIDER_COUNT} deterministic_uri_clone_reduction_percent=75.0000 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
    );
}

fn indexed_shader(
    locator: AssetUri,
    include_path: Option<&str>,
    imports: &[&str],
) -> IndexedShaderImports {
    IndexedShaderImports {
        locator,
        include_path: include_path.map(str::to_string),
        imports: imports.iter().map(|path| (*path).to_string()).collect(),
    }
}

fn legacy_provider_locators(
    imports: &[AssetId],
    providers: &HashMap<AssetId, AssetUri>,
) -> Vec<AssetUri> {
    let mut dependencies = Vec::with_capacity(imports.len());
    for provider_id in imports {
        let Some(locator) = providers.get(provider_id).cloned() else {
            continue;
        };
        if !dependencies.contains(&locator) {
            dependencies.push(locator);
        }
    }
    dependencies
}

fn hashed_provider_locators(
    imports: &[AssetId],
    providers: &HashMap<AssetId, AssetUri>,
) -> Vec<AssetUri> {
    let mut dependencies = Vec::with_capacity(imports.len());
    let mut seen_provider_ids = HashSet::with_capacity(imports.len());
    for provider_id in imports.iter().copied() {
        if !seen_provider_ids.insert(provider_id) {
            continue;
        }
        let Some(locator) = providers.get(&provider_id) else {
            continue;
        };
        dependencies.push(locator.clone());
    }
    dependencies
}

fn benchmark_paired_samples<const SAMPLE_COUNT: usize>(
    mut legacy: impl FnMut() -> Vec<AssetUri>,
    mut optimized: impl FnMut() -> Vec<AssetUri>,
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

fn benchmark_sample(operation: &mut impl FnMut() -> Vec<AssetUri>) -> u128 {
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
