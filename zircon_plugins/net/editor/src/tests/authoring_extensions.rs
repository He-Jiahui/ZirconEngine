use crate::{
    plugin_registration, NET_AUTHORING_CAPABILITY, NET_AUTHORING_VIEW_ID, NET_DIAGNOSTICS_VIEW_ID,
    NET_DRAWER_ID, NET_LISTENER_CONFIG_OPERATION, NET_REPLICATION_SCHEMA_ASSET_KIND,
    NET_REPLICATION_SCHEMA_CREATE_OPERATION, NET_REPLICATION_SCHEMA_PALETTE_ID,
    NET_REPLICATION_SCHEMA_TEMPLATE_ID, NET_ROUTE_CONFIG_OPERATION, NET_TEMPLATE_ID,
};

#[test]
fn net_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&NET_AUTHORING_CAPABILITY.to_string()));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == NET_AUTHORING_VIEW_ID));
    assert!(registration
        .extensions
        .views()
        .iter()
        .any(|view| view.id() == NET_DIAGNOSTICS_VIEW_ID));
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
        .any(|menu| menu.operation().as_str() == "view.net.authoring.open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "view.net.authoring.open"));
    assert!(registration
        .extensions
        .operations()
        .descriptors()
        .any(|operation| operation.path().as_str() == "view.net.diagnostics.open"));
    assert_operation_payload_schema(
        &registration,
        NET_LISTENER_CONFIG_OPERATION,
        "net.listener_config.v1",
    );
    assert_operation_payload_schema(
        &registration,
        NET_ROUTE_CONFIG_OPERATION,
        "net.route_config.v1",
    );
    assert_operation_payload_schema(
        &registration,
        NET_REPLICATION_SCHEMA_CREATE_OPERATION,
        "net.replication_schema.v1",
    );
    assert!(registration
        .extensions
        .component_drawers()
        .iter()
        .any(|drawer| drawer.component_type() == "net.ListenerConfig"));
    assert!(registration
        .extensions
        .component_drawers()
        .iter()
        .any(|drawer| drawer.component_type() == "net.HttpRouteConfig"));
    assert!(registration
        .extensions
        .component_drawers()
        .iter()
        .any(|drawer| drawer.component_type() == "net.ReplicationSchema"));
    assert!(registration
        .extensions
        .asset_creation_templates()
        .iter()
        .any(|template| template.id() == NET_REPLICATION_SCHEMA_TEMPLATE_ID));
    assert!(registration
        .extensions
        .graph_editors()
        .iter()
        .any(|editor| editor.asset_kind() == NET_REPLICATION_SCHEMA_ASSET_KIND));
    assert!(registration
        .extensions
        .graph_node_palettes()
        .iter()
        .any(|palette| palette.id() == NET_REPLICATION_SCHEMA_PALETTE_ID
            && palette.nodes().len() == 2));
}

fn assert_operation_payload_schema(
    registration: &zircon_editor::EditorPluginRegistrationReport,
    operation_path: &str,
    payload_schema_id: &str,
) {
    let operation = registration
        .extensions
        .operations()
        .descriptors()
        .find(|operation| operation.path().as_str() == operation_path)
        .expect("network authoring operation should be registered");
    assert_eq!(operation.payload_schema_id(), Some(payload_schema_id));
}
