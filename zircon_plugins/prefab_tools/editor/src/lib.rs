use std::collections::BTreeMap;

use zircon_editor::core::editor_authoring_extension::AssetCreationTemplateDescriptor;
use zircon_editor::core::editor_extension::{
    AssetEditorDescriptor, ComponentDrawerDescriptor, EditorExtensionRegistry,
    EditorMenuItemDescriptor,
};
use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, register_authoring_extensions,
    EditorAuthoringContributionBatch, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::asset::{PrefabInstanceAsset, PrefabPropertyOverrideAsset, TransformAsset};

pub const PLUGIN_ID: &str = zircon_plugin_prefab_tools_runtime::PLUGIN_ID;
pub const CAPABILITY: &str = "editor.extension.prefab_tools_authoring";
pub const PREFAB_AUTHORING_VIEW_ID: &str = "prefab_tools.authoring";
pub const PREFAB_DRAWER_ID: &str = "prefab_tools.drawer";
pub const PREFAB_TEMPLATE_ID: &str = "prefab_tools.authoring";

#[derive(Clone, Debug)]
pub struct PrefabToolsEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl PrefabToolsEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for PrefabToolsEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: PREFAB_DRAWER_ID,
                drawer_display_name: "Prefab Tools",
                template_id: PREFAB_TEMPLATE_ID,
                template_document: "plugins://prefab_tools/editor/authoring.ui.toml",
                surfaces: &[EditorAuthoringSurface::new(
                    PREFAB_AUTHORING_VIEW_ID,
                    "Prefabs",
                    "World",
                    "Plugins/Prefab Tools",
                )],
            },
        )?;
        register_authoring_contribution_batch(registry, prefab_authoring_batch())
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Prefab Tools",
        "zircon_plugin_prefab_tools_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> PrefabToolsEditorPlugin {
    PrefabToolsEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_prefab_tools_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_prefab_tools_runtime::package_manifest(),
    )
}

fn prefab_authoring_batch() -> EditorAuthoringContributionBatch {
    let create = operation("PrefabTools.Authoring.CreateFromSelection");
    let open = operation("PrefabTools.Authoring.Open");
    let apply = operation("PrefabTools.Authoring.ApplyOverrides");
    let revert = operation("PrefabTools.Authoring.RevertOverrides");
    let break_instance = operation("PrefabTools.Authoring.BreakInstance");
    EditorAuthoringContributionBatch {
        operations: vec![
            EditorOperationDescriptor::new(create.clone(), "Create Prefab From Selection")
                .with_menu_path("Plugins/Prefab Tools/Create From Selection")
                .with_payload_schema_id("prefab_tools.create_from_selection.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(open.clone(), "Open Prefab")
                .with_menu_path("Plugins/Prefab Tools/Open Prefab Asset")
                .with_payload_schema_id("prefab_tools.open_asset.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(apply.clone(), "Apply Prefab Overrides")
                .with_menu_path("Plugins/Prefab Tools/Apply Overrides")
                .with_payload_schema_id("prefab_tools.apply_overrides.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(revert.clone(), "Revert Prefab Overrides")
                .with_menu_path("Plugins/Prefab Tools/Revert Overrides")
                .with_payload_schema_id("prefab_tools.revert_overrides.v1")
                .with_required_capabilities([CAPABILITY]),
            EditorOperationDescriptor::new(break_instance.clone(), "Break Prefab Instance")
                .with_menu_path("Plugins/Prefab Tools/Break Instance")
                .with_payload_schema_id("prefab_tools.break_instance.v1")
                .with_required_capabilities([CAPABILITY]),
        ],
        menu_items: vec![
            menu_item("Plugins/Prefab Tools/Create From Selection", &create),
            menu_item("Plugins/Prefab Tools/Open Prefab Asset", &open),
            menu_item("Plugins/Prefab Tools/Apply Overrides", &apply),
            menu_item("Plugins/Prefab Tools/Revert Overrides", &revert),
            menu_item("Plugins/Prefab Tools/Break Instance", &break_instance),
        ],
        asset_editors: vec![AssetEditorDescriptor::new(
            "prefab.asset",
            PREFAB_AUTHORING_VIEW_ID,
            "Prefab",
            open,
        )
        .with_required_capabilities([CAPABILITY])],
        component_drawers: vec![ComponentDrawerDescriptor::new(
            zircon_plugin_prefab_tools_runtime::PREFAB_INSTANCE_COMPONENT_TYPE,
            "plugins://prefab_tools/editor/prefab_instance.ui.toml",
            "prefab_tools.editor.component",
        )],
        asset_creation_templates: vec![AssetCreationTemplateDescriptor::new(
            "prefab_tools.template.prefab",
            "Prefab",
            "prefab.asset",
            create,
        )
        .with_default_document("plugins://prefab_tools/templates/default_prefab.toml")
        .with_required_capabilities([CAPABILITY])],
        ..Default::default()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrokenPrefabInstanceAuthoringState {
    pub local_transform: TransformAsset,
    pub baked_overrides: Vec<PrefabPropertyOverrideAsset>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrefabOverrideApplication {
    pub applied_overrides: Vec<PrefabPropertyOverrideAsset>,
    pub cleared_instance_override_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefabOverrideRevertReport {
    pub reverted_override_count: usize,
}

pub fn effective_prefab_overrides(
    instance: &PrefabInstanceAsset,
) -> Vec<PrefabPropertyOverrideAsset> {
    let mut overrides = BTreeMap::new();
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

pub fn validate_prefab_instance(
    instance: &PrefabInstanceAsset,
    source_prefab_available: bool,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !source_prefab_available {
        diagnostics.push(format!(
            "prefab instance source `{}` is not available",
            instance.prefab
        ));
    }
    for override_value in &instance.overrides {
        if override_value.entity_path.trim().is_empty() {
            diagnostics.push("prefab override entity path must not be empty".to_string());
        }
        if override_value.property_path.trim().is_empty() {
            diagnostics.push("prefab override property path must not be empty".to_string());
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn apply_prefab_overrides(
    instance: &mut PrefabInstanceAsset,
    source_prefab_available: bool,
) -> Result<PrefabOverrideApplication, Vec<String>> {
    let diagnostics = validate_prefab_instance(instance, source_prefab_available);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let cleared_instance_override_count = instance.overrides.len();
    let applied_overrides = effective_prefab_overrides(instance);
    instance.overrides.clear();
    Ok(PrefabOverrideApplication {
        applied_overrides,
        cleared_instance_override_count,
    })
}

pub fn revert_prefab_overrides(instance: &mut PrefabInstanceAsset) -> PrefabOverrideRevertReport {
    let reverted_override_count = instance.overrides.len();
    instance.overrides.clear();
    PrefabOverrideRevertReport {
        reverted_override_count,
    }
}

pub fn break_prefab_instance(instance: &PrefabInstanceAsset) -> BrokenPrefabInstanceAuthoringState {
    BrokenPrefabInstanceAuthoringState {
        local_transform: instance.local_transform.clone(),
        baked_overrides: effective_prefab_overrides(instance),
    }
}

fn operation(path: &str) -> EditorOperationPath {
    EditorOperationPath::parse(path).expect("valid prefab operation path")
}

fn menu_item(path: &str, operation: &EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation.clone()).with_required_capabilities([CAPABILITY])
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_editor::EditorPlugin;
    use zircon_runtime::asset::{AssetReference, AssetUri};

    #[test]
    fn prefab_authoring_registration_exposes_menu_items_and_payload_schemas() {
        let mut registry = EditorExtensionRegistry::default();
        editor_plugin()
            .register_editor_extensions(&mut registry)
            .expect("prefab authoring registration");
        let operation = operation("PrefabTools.Authoring.ApplyOverrides");
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
        let instance = PrefabInstanceAsset {
            prefab: asset_ref("res://prefabs/ship.prefab.toml"),
            local_transform: TransformAsset::default(),
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
        let instance = PrefabInstanceAsset {
            prefab: asset_ref("res://prefabs/missing.prefab.toml"),
            local_transform: TransformAsset::default(),
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
        let mut instance = PrefabInstanceAsset {
            prefab: asset_ref("res://prefabs/ship.prefab.toml"),
            local_transform: TransformAsset::default(),
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
        let mut instance = PrefabInstanceAsset {
            prefab: asset_ref("res://prefabs/missing.prefab.toml"),
            local_transform: TransformAsset::default(),
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
        let mut instance = PrefabInstanceAsset {
            prefab: asset_ref("res://prefabs/ship.prefab.toml"),
            local_transform: TransformAsset::default(),
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
        let instance = PrefabInstanceAsset {
            prefab: asset_ref("res://prefabs/ship.prefab.toml"),
            local_transform: TransformAsset::default(),
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
}
