use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::resource::AssetUuid;

use super::*;
use crate::asset::assets::material::MaterialTextureSlotValue;
use crate::asset::AssetUri;

fn reference(uuid_label: &str, locator: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_stable_label(uuid_label),
        AssetUri::parse(locator).unwrap(),
    )
}

fn material() -> MaterialAsset {
    MaterialAsset::from_toml_str(
        r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .unwrap()
}

fn legacy_dedup(references: &[AssetReference]) -> Vec<AssetReference> {
    let mut unique = Vec::with_capacity(references.len());
    for reference in references {
        if !unique.contains(reference) {
            unique.push(reference.clone());
        }
    }
    unique
}

fn indexed_dedup(references: &[AssetReference]) -> Vec<AssetReference> {
    let mut seen = HashSet::with_capacity(references.len());
    let mut unique = Vec::with_capacity(references.len());
    for reference in references {
        if seen.insert(reference) {
            unique.push(reference.clone());
        }
    }
    unique
}

#[test]
fn optimization_batch_hy_runtime608_preserves_projection_specific_texture_equality() {
    let shared_locator = "res://textures/shared.png";
    let first = reference("first-uuid", shared_locator);
    let second = reference("second-uuid", shared_locator);
    let mut material = material();
    material.texture_slots.insert(
        "custom_a".to_string(),
        MaterialTextureSlotValue::new(first.clone()),
    );
    material
        .texture_slots
        .insert("custom_b".to_string(), MaterialTextureSlotValue::new(first));
    material.texture_slots.insert(
        "custom_c".to_string(),
        MaterialTextureSlotValue::new(second),
    );

    let references = direct_references(&material);
    let locators = material.direct_reference_locators();

    assert_eq!(references.len(), 3, "shader plus two distinct UUIDs");
    assert_eq!(locators.len(), 2, "shader plus one shared locator");
}

#[test]
fn optimization_batch_hy_runtime608_texture_dedup_uses_borrowed_hash_keys() {
    let source = include_str!("../../dependency_set.rs");
    let helper = source
        .split("fn collect_direct_references")
        .nth(1)
        .expect("direct reference collector")
        .split("#[cfg(test)]")
        .next()
        .expect("bounded direct reference collector");

    assert!(source.contains("HashSet::with_capacity(texture_slots.len())"));
    assert!(helper.contains("texture_keys.insert(key(texture))"));
    assert!(!helper.contains("references[1..].contains(&texture)"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hy_runtime608_borrowed_texture_dedup_performance_evidence() {
    const TEXTURE_COUNT: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;
    let references = (0..TEXTURE_COUNT)
        .map(|index| {
            reference(
                &format!("texture-{index:05}"),
                &format!("res://textures/performance_{index:05}.png"),
            )
        })
        .collect::<Vec<_>>();
    let measure_legacy = || {
        let started = Instant::now();
        black_box(legacy_dedup(black_box(&references)));
        started.elapsed().as_nanos().max(1)
    };
    let measure_indexed = || {
        let started = Instant::now();
        black_box(indexed_dedup(black_box(&references)));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_indexed());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    indexed_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let indexed_p50 = indexed_samples[8];
    let indexed_p95 = indexed_samples[16];
    println!(
        "RUNTIME608_BORROWED_TEXTURE_DEDUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 indexed_first_pairs=8 textures={TEXTURE_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} indexed_p50_ns={indexed_p50} indexed_p95_ns={indexed_p95} equality_probes_worst_case=8386560->4096 target_ratio_bp=1000"
    );
    assert!(
        indexed_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(1_000),
        "borrowed texture dedup P95 {indexed_p95} ns exceeded 10% of legacy {legacy_p95} ns"
    );
}
