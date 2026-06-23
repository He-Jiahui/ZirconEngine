use zircon_runtime::builtin::RuntimeTargetMode;

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
