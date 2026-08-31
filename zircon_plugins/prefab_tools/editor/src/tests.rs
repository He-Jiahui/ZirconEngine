use super::*;
use std::hint::black_box;
use std::time::{Duration, Instant};
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::asset::{AssetReference, AssetUri, PrefabPropertyOverrideAsset};

#[test]
fn prefab_authoring_registration_exposes_only_read_only_surfaces_without_factories() {
    let mut registry = EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("prefab authoring registration");
    let operations = [
        "prefab_tools.authoring.create_from_selection",
        "prefab_tools.authoring.open",
        "prefab_tools.authoring.apply_overrides",
        "prefab_tools.authoring.revert_overrides",
        "prefab_tools.authoring.break_instance",
    ];

    for operation in operations {
        let operation = EditorOperationPath::parse(operation).expect("operation path");
        assert!(registry.commands().command(&operation).is_none());
    }
    let unavailable_operations = [
        "prefab_tools.authoring.create_from_selection",
        "prefab_tools.authoring.open",
        "prefab_tools.authoring.apply_overrides",
        "prefab_tools.authoring.revert_overrides",
        "prefab_tools.authoring.break_instance",
    ];
    assert!(registry.menu_items().iter().all(|item| {
        !unavailable_operations
            .iter()
            .any(|operation| item.operation().as_str() == *operation)
    }));
    assert!(registry.asset_type_contributions().is_empty());
    assert!(registry
        .views()
        .iter()
        .any(|view| view.id() == PREFAB_AUTHORING_VIEW_ID));
    assert!(registry
        .inspector_customizations()
        .iter()
        .any(|customization| {
            customization.id() == zircon_plugin_prefab_tools_runtime::PREFAB_INSTANCE_COMPONENT_TYPE
        }));
}

#[test]
fn prefab_override_precedence_keeps_last_override_for_same_property() {
    let instance = zircon_runtime::asset::PrefabInstanceAsset {
        prefab: asset_ref("res://prefabs/ship.prefab.toml"),
        local_transform: zircon_runtime::asset::TransformAsset::default(),
        overrides: vec![
            override_value("root", "Transform.translation.x", 1),
            override_value("root", "Transform.translation.x", 2),
            override_value("root", "Transform.translation.y", 3),
        ],
    };

    let effective = effective_prefab_overrides(&instance);

    assert_eq!(effective.len(), 2);
    assert!(effective.iter().any(|override_value| {
        override_value.property_path == "Transform.translation.x"
            && override_value.value == serde_json::json!(2)
    }));
}

#[test]
fn prefab_instance_validation_rejects_duplicate_override_paths() {
    let instance = zircon_runtime::asset::PrefabInstanceAsset {
        prefab: asset_ref("res://prefabs/ship.prefab.toml"),
        local_transform: zircon_runtime::asset::TransformAsset::default(),
        overrides: vec![
            override_value("root", "Transform.translation.x", 1),
            override_value("root", "Transform.translation.x", 2),
        ],
    };

    let diagnostics = validate_prefab_instance(&instance, true);

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].contains("duplicate prefab override"));
    assert!(diagnostics[0].contains("root"));
    assert!(diagnostics[0].contains("Transform.translation.x"));
}

#[test]
fn prefab_instance_validation_reports_missing_source_and_bad_override_paths() {
    let instance = zircon_runtime::asset::PrefabInstanceAsset {
        prefab: asset_ref("res://prefabs/missing.prefab.toml"),
        local_transform: zircon_runtime::asset::TransformAsset::default(),
        overrides: vec![override_value(" ", " ", 1)],
    };

    let diagnostics = validate_prefab_instance(&instance, false);

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("source `res://prefabs/missing.prefab.toml`")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("entity path must not be empty")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("property path must not be empty")));
}

#[test]
#[ignore = "release performance gate"]
fn prefab_effective_override_release_gate_uses_borrowed_index_keys() {
    const SAMPLE_PAIRS: usize = 21;
    const OVERRIDE_COUNT: usize = 8_192;
    const QUERIES_PER_SAMPLE: usize = 8;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

    let instance = large_prefab_instance(OVERRIDE_COUNT);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_effective_override_batch(
                &instance,
                QUERIES_PER_SAMPLE,
                legacy_effective_prefab_overrides,
            ));
            optimized_samples.push(measure_effective_override_batch(
                &instance,
                QUERIES_PER_SAMPLE,
                effective_prefab_overrides,
            ));
        } else {
            optimized_samples.push(measure_effective_override_batch(
                &instance,
                QUERIES_PER_SAMPLE,
                effective_prefab_overrides,
            ));
            legacy_samples.push(measure_effective_override_batch(
                &instance,
                QUERIES_PER_SAMPLE,
                legacy_effective_prefab_overrides,
            ));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples).as_nanos();
    let optimized_p95 = nearest_rank_p95(&optimized_samples).as_nanos();
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins08_prefab_borrowed_override_index sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even override_count={OVERRIDE_COUNT} queries_per_sample={QUERIES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}",
        durations_csv(&legacy_samples),
        durations_csv(&optimized_samples)
    );
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "borrowed override keys must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn override_value(
    entity_path: &str,
    property_path: &str,
    value: i32,
) -> PrefabPropertyOverrideAsset {
    PrefabPropertyOverrideAsset {
        entity_path: entity_path.to_string(),
        property_path: property_path.to_string(),
        value: serde_json::json!(value),
    }
}

fn asset_ref(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}

fn large_prefab_instance(override_count: usize) -> zircon_runtime::asset::PrefabInstanceAsset {
    let overrides = (0..override_count)
        .map(|index| PrefabPropertyOverrideAsset {
            entity_path: format!(
                "root/sector_{:08}/module_{:08}/component_{:08}",
                index / 256,
                index / 16,
                index
            ),
            property_path: format!(
                "Transform.translation.axis_{:08}.channel_{:08}",
                index % 3,
                index
            ),
            value: serde_json::json!({ "index": index, "enabled": true }),
        })
        .collect();
    zircon_runtime::asset::PrefabInstanceAsset {
        prefab: asset_ref("res://prefabs/large.prefab.toml"),
        local_transform: zircon_runtime::asset::TransformAsset::default(),
        overrides,
    }
}

fn legacy_effective_prefab_overrides(
    instance: &zircon_runtime::asset::PrefabInstanceAsset,
) -> Vec<PrefabPropertyOverrideAsset> {
    let mut overrides = std::collections::BTreeMap::new();
    for override_value in &instance.overrides {
        overrides.insert(
            (
                override_value.entity_path.clone(),
                override_value.property_path.clone(),
            ),
            override_value.clone(),
        );
    }
    overrides.into_values().collect()
}

fn measure_effective_override_batch(
    instance: &zircon_runtime::asset::PrefabInstanceAsset,
    queries_per_sample: usize,
    query: fn(&zircon_runtime::asset::PrefabInstanceAsset) -> Vec<PrefabPropertyOverrideAsset>,
) -> Duration {
    let started = Instant::now();
    let mut output_count = 0usize;
    for _ in 0..queries_per_sample {
        output_count += black_box(query(black_box(instance))).len();
    }
    let elapsed = started.elapsed();
    assert_eq!(output_count, instance.overrides.len() * queries_per_sample);
    black_box(output_count);
    elapsed
}

fn nearest_rank_p95(samples: &[Duration]) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[((sorted.len() * 95).div_ceil(100)).saturating_sub(1)]
}

fn durations_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}
