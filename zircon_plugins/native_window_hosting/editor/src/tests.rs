use zircon_runtime::{
    builtin::RuntimeTargetMode,
    plugin::{ExportPackagingStrategy, PluginModuleKind},
};

use super::*;

#[test]
fn native_window_hosting_plugin_contributes_window_views_and_capability() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration.capabilities.contains(&CAPABILITY.to_string()));
    assert_eq!(registration.package_manifest.category, "platform");
    assert_eq!(
        registration.package_manifest.supported_targets,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(
        registration.package_manifest.capabilities,
        vec![CAPABILITY.to_string()]
    );
    let views = registration.extensions.views();
    assert!(views
        .iter()
        .any(|view| view.id() == WORKBENCH_WINDOW_VIEW_ID));
    assert!(views.iter().any(|view| view.id() == PREFAB_WINDOW_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == NATIVE_WINDOW_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == NATIVE_WINDOW_TEMPLATE_ID));
    for operation_path in [
        "View.editor.workbench_window.Open",
        "View.editor.prefab.Open",
    ] {
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == operation_path));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == operation_path));
    }
}

#[test]
fn native_window_hosting_package_manifest_declares_editor_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("native_window_hosting declares standalone distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(
        distribution.dist_crate,
        NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME
    );
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert!(distribution.runtime_entry.is_empty());
    assert_eq!(
        distribution.editor_entry,
        NATIVE_WINDOW_HOSTING_DIST_EDITOR_ENTRY
    );

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "native_window_hosting.dist")
        .expect("native_window_hosting dist module is declared");
    assert_eq!(dist_module.kind, PluginModuleKind::Native);
    assert_eq!(
        dist_module.crate_name,
        NATIVE_WINDOW_HOSTING_DIST_CRATE_NAME
    );
    assert_eq!(
        dist_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(dist_module.capabilities, vec![CAPABILITY.to_string()]);
}
