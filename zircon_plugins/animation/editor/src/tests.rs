use super::*;

#[test]
fn animation_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&ANIMATION_AUTHORING_CAPABILITY.to_string()));
    assert_eq!(
        editor_plugin().declaration().mirrored_runtime_package_id(),
        Some(PLUGIN_ID)
    );
    assert!(registration
        .package_manifest
        .capabilities
        .contains(&zircon_plugin_animation_runtime::ANIMATION_RUNTIME_CAPABILITY.to_string()));
    assert!(registration
        .package_manifest
        .capabilities
        .contains(&ANIMATION_AUTHORING_CAPABILITY.to_string()));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == ANIMATION_AUTHORING_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == ANIMATION_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == ANIMATION_TEMPLATE_ID));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "View.animation.authoring.Open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "View.animation.authoring.Open"));
}
