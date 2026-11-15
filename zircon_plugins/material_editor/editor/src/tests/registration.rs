use crate::{
    editor_plugin, package_manifest, CAPABILITY, MATERIAL_EDITOR_DIST_CRATE_NAME,
    MATERIAL_EDITOR_DIST_EDITOR_ENTRY,
};
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::PluginModuleKind;

#[test]
fn material_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("material authoring registration");
    let operation =
        EditorOperationPath::parse("material_editor.graph.compile").expect("valid material path");
    let descriptor = registry
        .pending_command(&operation)
        .expect("compile operation registered");

    assert_eq!(
        descriptor
            .menu_path()
            .expect("compile command menu path")
            .stable_path(),
        "plugins/material_editor/material_editor.graph.compile"
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("material_editor.compile_graph.v1")
    );
    assert!(registry.menu_items().is_empty());
}

#[test]
fn material_editor_package_manifest_declares_editor_only_metadata() {
    let manifest = package_manifest();
    let editor_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Editor)
        .expect("material editor module");

    assert_eq!(manifest.category, "authoring");
    assert_eq!(
        manifest.supported_targets,
        vec![zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost]
    );
    assert_eq!(manifest.capabilities, vec![CAPABILITY.to_string()]);
    assert_eq!(editor_module.capabilities, manifest.capabilities);
}

#[test]
fn material_editor_package_manifest_declares_editor_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("material_editor declares standalone distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, MATERIAL_EDITOR_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert!(distribution.runtime_entry.is_empty());
    assert_eq!(distribution.editor_entry, MATERIAL_EDITOR_DIST_EDITOR_ENTRY);

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "material_editor.dist")
        .expect("material_editor dist module is declared");
    assert_eq!(dist_module.kind, PluginModuleKind::Native);
    assert_eq!(dist_module.crate_name, MATERIAL_EDITOR_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(dist_module.capabilities, vec![CAPABILITY.to_string()]);
}
