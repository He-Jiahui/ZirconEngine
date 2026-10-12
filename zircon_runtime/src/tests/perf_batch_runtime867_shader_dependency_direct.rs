use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use crate::asset::{AssetId, AssetUri};

use super::{IndexedShaderImports, ShaderImportDependencyIndex};

const IMPORT_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn runtime867_shader_dependency_direct_append_preserves_metadata_and_provider_order() {
    let consumer_id = AssetId::new();
    let first_provider_id = AssetId::new();
    let second_provider_id = AssetId::new();
    let first_locator = AssetUri::parse("res://runtime867/includes/first.zshader")
        .expect("valid first provider URI");
    let second_locator = AssetUri::parse("res://runtime867/includes/second.zshader")
        .expect("valid second provider URI");
    let mut index = ShaderImportDependencyIndex::default();
    index.shaders_by_id.insert(
        first_provider_id,
        indexed_shader(first_locator.clone(), Some("runtime867.first"), &[]),
    );
    index.shaders_by_id.insert(
        second_provider_id,
        indexed_shader(second_locator.clone(), Some("runtime867.second"), &[]),
    );
    index.shaders_by_id.insert(
        consumer_id,
        indexed_shader(
            AssetUri::parse("res://runtime867/consumer.zshader").expect("valid consumer URI"),
            None,
            &["runtime867.second", "runtime867.first", "runtime867.second"],
        ),
    );
    index.includes_by_path.insert(
        "runtime867.first".to_string(),
        HashSet::from([first_provider_id]),
    );
    index.includes_by_path.insert(
        "runtime867.second".to_string(),
        HashSet::from([second_provider_id]),
    );

    let mut dependencies_by_id = HashMap::from([(consumer_id, vec![second_locator.clone()])]);
    index.append_dependencies(&mut dependencies_by_id);

    let dependencies = dependencies_by_id
        .get(&consumer_id)
        .expect("consumer dependency output must remain present");
    assert_eq!(
        dependencies,
        &[
            second_locator.clone(),
            second_locator.clone(),
            first_locator.clone(),
        ]
    );
    assert!(dependencies.capacity() >= 4);
    assert_eq!(
        index.dependency_locators(consumer_id),
        vec![second_locator, first_locator]
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime867_shader_dependency_direct_append_benchmark() {
    let seed = AssetUri::parse("res://runtime867/metadata-owned.zshader")
        .expect("valid metadata-owned URI");
    let locators = (0..IMPORT_COUNT)
        .map(|index| {
            AssetUri::parse(&format!(
                "res://runtime867/includes/provider-{index:04}-long-module-name.zshader"
            ))
            .expect("valid provider URI")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_append(&seed, &locators),
        optimized_append(&seed, &locators)
    );

    for _ in 0..4 {
        black_box(legacy_append(&seed, &locators));
        black_box(optimized_append(&seed, &locators));
    }

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_append(&seed, &locators)));
            optimized.push(measure(|| optimized_append(&seed, &locators)));
        } else {
            optimized.push(measure(|| optimized_append(&seed, &locators)));
            legacy.push(measure(|| legacy_append(&seed, &locators)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "RUNTIME867_SHADER_DEPENDENCY_DIRECT_APPEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} imports={IMPORT_COUNT} legacy_temporary_locator_slots={IMPORT_COUNT} optimized_temporary_locator_slots=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95 <= legacy_p95,
        "direct append P95 {optimized_p95}ns must not exceed temporary projection P95 {legacy_p95}ns"
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

fn legacy_append(seed: &AssetUri, locators: &[AssetUri]) -> Vec<AssetUri> {
    let mut dependencies = vec![seed.clone()];
    let projected = locators.to_vec();
    for locator in projected {
        dependencies.push(locator);
    }
    dependencies
}

fn optimized_append(seed: &AssetUri, locators: &[AssetUri]) -> Vec<AssetUri> {
    let mut dependencies = vec![seed.clone()];
    dependencies.reserve(locators.len());
    for locator in locators {
        dependencies.push(locator.clone());
    }
    dependencies
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
