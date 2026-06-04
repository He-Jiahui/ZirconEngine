use crate::{plugin_registration, NET_AUTHORING_VIEW_ID, NET_DRAWER_ID, NET_TEMPLATE_ID};

#[test]
fn net_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&"editor.extension.net_authoring".to_string()));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == NET_AUTHORING_VIEW_ID));
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == NET_DRAWER_ID));
    assert!(registration
        .extensions
        .ui_templates()
        .iter()
        .any(|template| template.id() == NET_TEMPLATE_ID));
    assert!(registration
        .extensions
        .menu_items()
        .iter()
        .any(|menu| menu.operation().as_str() == "View.net.authoring.Open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "View.net.authoring.Open"));
}
