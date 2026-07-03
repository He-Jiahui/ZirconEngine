use super::*;

#[test]
fn hybrid_gi_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&HYBRID_GI_AUTHORING_CAPABILITY.to_string()));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == HYBRID_GI_AUTHORING_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == HYBRID_GI_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == HYBRID_GI_TEMPLATE_ID));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "view.hybrid_gi.authoring.open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "view.hybrid_gi.authoring.open"));
}
