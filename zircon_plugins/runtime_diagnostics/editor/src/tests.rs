use zircon_runtime::builtin::RuntimeTargetMode;

use super::*;

#[test]
fn runtime_diagnostics_plugin_contributes_view_and_capability() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration.capabilities.contains(&CAPABILITY.to_string()));
    assert_eq!(registration.package_manifest.category, "diagnostics");
    assert_eq!(
        registration.package_manifest.supported_targets,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(
        registration.package_manifest.capabilities,
        vec![CAPABILITY.to_string()]
    );
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == RUNTIME_DIAGNOSTICS_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == RUNTIME_DIAGNOSTICS_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == RUNTIME_DIAGNOSTICS_TEMPLATE_ID));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "View.editor.runtime_diagnostics.Open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "View.editor.runtime_diagnostics.Open"));
}
