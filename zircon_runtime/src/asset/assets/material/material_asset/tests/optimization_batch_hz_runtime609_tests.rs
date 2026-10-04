use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::resource::AssetUuid;

use super::*;
use crate::asset::AssetUri;

fn reference(label: &str, locator: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_stable_label(label),
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

fn legacy_merge(schema_slots: &[String], custom_slots: &[String]) -> Vec<String> {
    let mut slots = schema_slots.to_vec();
    for slot in custom_slots {
        if !slots.iter().any(|existing| existing == slot) {
            slots.push(slot.clone());
        }
    }
    slots
}

fn bounded_schema_merge(schema_slots: &[String], custom_slots: &[String]) -> Vec<String> {
    let mut slots = schema_slots.to_vec();
    let schema_slot_count = slots.len();
    for slot in custom_slots {
        if !slots[..schema_slot_count]
            .iter()
            .any(|existing| existing == slot)
        {
            slots.push(slot.clone());
        }
    }
    slots
}

#[test]
fn optimization_batch_hz_runtime609_preserves_schema_precedence_and_custom_slot_order() {
    let schema_reference = reference("schema-base", "res://textures/schema-base.png");
    let shadowed_reference = reference("shadowed-base", "res://textures/shadowed-base.png");
    let mut material = material();
    material.base_color_texture = Some(schema_reference.clone());
    material.texture_slots.insert(
        "base_color_texture".to_string(),
        MaterialTextureSlotValue::new(shadowed_reference),
    );
    material.texture_slots.insert(
        "custom_b".to_string(),
        MaterialTextureSlotValue::new(reference("custom-b", "res://textures/custom-b.png")),
    );
    material.texture_slots.insert(
        "custom_a".to_string(),
        MaterialTextureSlotValue::new(reference("custom-a", "res://textures/custom-a.png")),
    );

    let slots = material.all_texture_slots();

    assert_eq!(
        slots
            .iter()
            .map(|(slot, _)| slot.as_str())
            .collect::<Vec<_>>(),
        vec!["base_color_texture", "custom_a", "custom_b"]
    );
    assert_eq!(slots[0].1, &schema_reference);

    let mut custom_only = self::material();
    let custom_schema_name = reference("custom-schema-name", "res://textures/custom-schema.png");
    custom_only.texture_slots.insert(
        "base_color_texture".to_string(),
        MaterialTextureSlotValue::new(custom_schema_name.clone()),
    );
    let slots = custom_only.all_texture_slots();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].0, "base_color_texture");
    assert_eq!(slots[0].1, &custom_schema_name);
}

#[test]
fn optimization_batch_hz_runtime609_limits_duplicate_checks_to_schema_slots() {
    let source = include_str!("../../material_asset.rs");
    let collector = source
        .split("pub fn all_texture_slots")
        .nth(1)
        .expect("texture-slot collector")
        .split("fn schema_v1_pbr_texture_slots")
        .next()
        .expect("bounded texture-slot collector");

    assert!(collector.contains("let schema_slot_count = slots.len();"));
    assert!(collector.contains("slots[..schema_slot_count]"));
    assert!(!collector.contains("if !slots.iter().any"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hz_runtime609_bounded_schema_slot_scan_performance_evidence() {
    const CUSTOM_SLOTS: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;
    let schema_slots = [
        "base_color_texture",
        "normal_texture",
        "metallic_roughness_texture",
        "occlusion_texture",
        "emissive_texture",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    let custom_slots = (0..CUSTOM_SLOTS)
        .map(|index| format!("custom_performance_slot_{index:05}"))
        .collect::<Vec<_>>();
    let measure_legacy = || {
        let started = Instant::now();
        black_box(legacy_merge(
            black_box(&schema_slots),
            black_box(&custom_slots),
        ));
        started.elapsed().as_nanos().max(1)
    };
    let measure_bounded = || {
        let started = Instant::now();
        black_box(bounded_schema_merge(
            black_box(&schema_slots),
            black_box(&custom_slots),
        ));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_bounded());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut bounded_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            bounded_samples.push(measure_bounded());
        } else {
            bounded_samples.push(measure_bounded());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    bounded_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let bounded_p50 = bounded_samples[8];
    let bounded_p95 = bounded_samples[16];
    println!(
        "RUNTIME609_BOUNDED_SCHEMA_SLOT_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 bounded_first_pairs=8 custom_slots={CUSTOM_SLOTS} schema_slots=5 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} bounded_p50_ns={bounded_p50} bounded_p95_ns={bounded_p95} duplicate_comparisons_worst_case=8407040->20480 target_ratio_bp=1000"
    );
    assert!(
        bounded_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(1_000),
        "bounded schema scan P95 {bounded_p95} ns exceeded 10% of legacy {legacy_p95} ns"
    );
}
