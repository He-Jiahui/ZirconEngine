use zircon_runtime::plugin::{ExportPackagingStrategy, PluginModuleKind};

use crate::capability::{ASSET_FIXTURE_CAPABILITY, CAPABILITY, WINDOW_CAPABILITY};
use crate::extension_ids::{
    ASSET_INSPECTOR_VIEW_ID, MODEL_ASSET_KIND, MODEL_IMPORTER_ID, MODEL_IMPORT_SETTINGS_COMPONENT,
    MODEL_IMPORT_SETTINGS_TEMPLATE_ID, WINDOW_VIEW_ID,
};
use crate::plugin::{package_manifest, plugin_registration};

#[test]
fn sdk_examples_package_contributes_window_importer_and_inspector() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert_eq!(
        registration.capabilities,
        vec![
            CAPABILITY.to_string(),
            WINDOW_CAPABILITY.to_string(),
            ASSET_FIXTURE_CAPABILITY.to_string()
        ]
    );
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == WINDOW_VIEW_ID));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == ASSET_INSPECTOR_VIEW_ID));
    let importer = registration
        .extensions
        .asset_importers()
        .into_iter()
        .find(|importer| importer.id() == MODEL_IMPORTER_ID)
        .expect("SDK model importer");
    assert_eq!(
        importer.source_extensions(),
        &["glb".to_string(), "gltf".to_string()]
    );
    assert_eq!(importer.output_kind(), Some(MODEL_ASSET_KIND));
    assert_eq!(importer.priority(), 10);
    assert!(registration
        .extensions
        .asset_editors()
        .iter()
        .any(|editor| editor.asset_kind() == MODEL_ASSET_KIND
            && editor.view_id() == ASSET_INSPECTOR_VIEW_ID));
    assert!(registration
        .extensions
        .component_drawers()
        .iter()
        .any(|drawer| drawer.component_type() == MODEL_IMPORT_SETTINGS_COMPONENT));
    assert!(registration
        .extensions
        .asset_creation_templates()
        .iter()
        .any(|template| template.id() == MODEL_IMPORT_SETTINGS_TEMPLATE_ID));
}

#[test]
fn sdk_examples_package_manifest_declares_sdk_fixture_metadata() {
    let manifest = package_manifest();

    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "sdk");
    assert_eq!(
        manifest.supported_targets,
        vec![zircon_runtime::builtin::RuntimeTargetMode::EditorHost]
    );
    assert!(manifest.capabilities.contains(&CAPABILITY.to_string()));
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Editor
            && module.crate_name == "zircon_plugin_sdk_examples_editor"));
    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::SourceTemplate));
}
