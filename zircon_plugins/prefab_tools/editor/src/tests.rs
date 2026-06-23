use super::*;
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::asset::{AssetReference, AssetUri, PrefabPropertyOverrideAsset};

#[test]
fn prefab_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("prefab authoring registration");
    let operation =
        EditorOperationPath::parse("PrefabTools.Authoring.ApplyOverrides").expect("operation path");
    let descriptor = registry
        .operations()
        .descriptor(&operation)
        .expect("apply overrides operation registered");

    assert_eq!(
        descriptor.menu_path(),
        Some("Plugins/Prefab Tools/Apply Overrides")
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("prefab_tools.apply_overrides.v1")
    );
    assert!(registry.menu_items().iter().any(|item| {
        item.path() == "Plugins/Prefab Tools/Apply Overrides" && item.operation() == &operation
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
fn apply_prefab_overrides_bakes_effective_values_and_clears_instance() {
    let mut instance = zircon_runtime::asset::PrefabInstanceAsset {
        prefab: asset_ref("res://prefabs/ship.prefab.toml"),
        local_transform: zircon_runtime::asset::TransformAsset::default(),
        overrides: vec![
            override_value("root", "Transform.translation.x", 1),
            override_value("root", "Transform.translation.x", 2),
            override_value("root", "Transform.translation.y", 3),
        ],
    };

    let report = apply_prefab_overrides(&mut instance, true)
        .expect("source prefab is available and overrides are valid");

    assert_eq!(report.cleared_instance_override_count, 3);
    assert_eq!(report.applied_overrides.len(), 2);
    assert!(instance.overrides.is_empty());
    assert!(report.applied_overrides.iter().any(|override_value| {
        override_value.property_path == "Transform.translation.x"
            && override_value.value == serde_json::json!(2)
    }));
}

#[test]
fn apply_prefab_overrides_keeps_instance_when_source_is_missing() {
    let mut instance = zircon_runtime::asset::PrefabInstanceAsset {
        prefab: asset_ref("res://prefabs/missing.prefab.toml"),
        local_transform: zircon_runtime::asset::TransformAsset::default(),
        overrides: vec![override_value("root", "Transform.translation.x", 1)],
    };

    let diagnostics = apply_prefab_overrides(&mut instance, false)
        .expect_err("missing source prefab blocks apply");

    assert_eq!(instance.overrides.len(), 1);
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("source `res://prefabs/missing.prefab.toml`")));
}

#[test]
fn revert_prefab_overrides_clears_instance_without_baking_values() {
    let mut instance = zircon_runtime::asset::PrefabInstanceAsset {
        prefab: asset_ref("res://prefabs/ship.prefab.toml"),
        local_transform: zircon_runtime::asset::TransformAsset::default(),
        overrides: vec![
            override_value("root", "Transform.translation.x", 1),
            override_value("root", "Transform.translation.y", 2),
        ],
    };

    let report = revert_prefab_overrides(&mut instance);

    assert_eq!(report.reverted_override_count, 2);
    assert!(instance.overrides.is_empty());
}

#[test]
fn break_prefab_instance_bakes_overrides_without_retaining_prefab_link() {
    let instance = zircon_runtime::asset::PrefabInstanceAsset {
        prefab: asset_ref("res://prefabs/ship.prefab.toml"),
        local_transform: zircon_runtime::asset::TransformAsset::default(),
        overrides: vec![
            override_value("root", "Transform.translation.x", 1),
            override_value("root", "Transform.translation.x", 2),
        ],
    };

    let broken = break_prefab_instance(&instance);

    assert_eq!(broken.baked_overrides.len(), 1);
    assert_eq!(
        broken.baked_overrides[0].value,
        serde_json::json!(2),
        "latest override should be the baked value"
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
