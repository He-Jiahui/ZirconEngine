use zircon_editor::core::editor_authoring_extension::{
    AssetCreationTemplateDescriptor, GraphEditorDescriptor, GraphNodeDescriptor,
    GraphNodePaletteDescriptor, GraphPinDescriptor,
};
use zircon_editor::core::editor_extension::{
    ComponentDrawerDescriptor, EditorExtensionRegistry, EditorExtensionRegistryError,
    EditorMenuItemDescriptor,
};
use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
use zircon_plugin_editor_support::{
    register_authoring_contribution_batch, EditorAuthoringContributionBatch, EditorAuthoringSurface,
};

use crate::capability::NET_AUTHORING_CAPABILITY;

pub const NET_AUTHORING_VIEW_ID: &str = "net.authoring";
pub const NET_DIAGNOSTICS_VIEW_ID: &str = "net.diagnostics";
pub const NET_DRAWER_ID: &str = "net.drawer";
pub const NET_TEMPLATE_ID: &str = "net.authoring";
pub const NET_REPLICATION_SCHEMA_ASSET_KIND: &str = "net.replication_schema";
pub const NET_REPLICATION_SCHEMA_TEMPLATE_ID: &str = "net.replication_schema.template";
pub const NET_REPLICATION_SCHEMA_PALETTE_ID: &str = "net.replication_schema.palette";
pub const NET_LISTENER_CONFIG_OPERATION: &str = "net.listener.configure";
pub const NET_ROUTE_CONFIG_OPERATION: &str = "net.route.configure";
pub const NET_REPLICATION_SCHEMA_OPEN_OPERATION: &str = "net.replication_schema.open";
pub const NET_REPLICATION_SCHEMA_VALIDATE_OPERATION: &str = "net.replication_schema.validate";
pub const NET_REPLICATION_SCHEMA_COMPILE_OPERATION: &str = "net.replication_schema.compile";
pub const NET_REPLICATION_SCHEMA_CREATE_OPERATION: &str = "net.replication_schema.create";

pub const NET_AUTHORING_SURFACES: &[EditorAuthoringSurface<'static>] = &[
    EditorAuthoringSurface::new(
        NET_AUTHORING_VIEW_ID,
        "Network",
        "Networking",
        "Plugins/Network",
    ),
    EditorAuthoringSurface::new(
        NET_DIAGNOSTICS_VIEW_ID,
        "Network Diagnostics",
        "Diagnostics",
        "Plugins/Network/Diagnostics",
    ),
];

pub fn register_net_authoring_workflows(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    let listener_config = operation_path(NET_LISTENER_CONFIG_OPERATION)?;
    let route_config = operation_path(NET_ROUTE_CONFIG_OPERATION)?;
    let schema_open = operation_path(NET_REPLICATION_SCHEMA_OPEN_OPERATION)?;
    let schema_validate = operation_path(NET_REPLICATION_SCHEMA_VALIDATE_OPERATION)?;
    let schema_compile = operation_path(NET_REPLICATION_SCHEMA_COMPILE_OPERATION)?;
    let schema_create = operation_path(NET_REPLICATION_SCHEMA_CREATE_OPERATION)?;

    register_authoring_contribution_batch(
        registry,
        EditorAuthoringContributionBatch {
            operations: vec![
                configure_operation(
                    listener_config.clone(),
                    "Configure Network Listener",
                    "Plugins/Network/Listener",
                    "net.listener_config.v1",
                ),
                configure_operation(
                    route_config.clone(),
                    "Configure Network Route",
                    "Plugins/Network/Route",
                    "net.route_config.v1",
                ),
                EditorOperationDescriptor::new(
                    schema_open.clone(),
                    "Open Network Replication Schema",
                )
                .with_required_capabilities([NET_AUTHORING_CAPABILITY]),
                EditorOperationDescriptor::new(
                    schema_validate.clone(),
                    "Validate Network Replication Schema",
                )
                .with_required_capabilities([NET_AUTHORING_CAPABILITY]),
                EditorOperationDescriptor::new(
                    schema_compile.clone(),
                    "Compile Network Replication Schema",
                )
                .with_required_capabilities([NET_AUTHORING_CAPABILITY]),
                EditorOperationDescriptor::new(
                    schema_create.clone(),
                    "Create Network Replication Schema",
                )
                .with_menu_path("Plugins/Network/Replication Schema")
                .with_payload_schema_id("net.replication_schema.v1")
                .with_required_capabilities([NET_AUTHORING_CAPABILITY]),
            ],
            menu_items: vec![
                menu_item("Plugins/Network/Listener", listener_config.clone()),
                menu_item("Plugins/Network/Route", route_config.clone()),
                menu_item("Plugins/Network/Replication Schema", schema_create.clone()),
            ],
            component_drawers: vec![
                ComponentDrawerDescriptor::new(
                    "net.ListenerConfig",
                    "plugins://net/editor/listener_config.zui",
                    "net.editor.listener_config",
                )
                .with_template_id(NET_TEMPLATE_ID)
                .with_data_root("net.listener")
                .with_binding(listener_config.as_str()),
                ComponentDrawerDescriptor::new(
                    "net.HttpRouteConfig",
                    "plugins://net/editor/route_config.zui",
                    "net.editor.route_config",
                )
                .with_template_id(NET_TEMPLATE_ID)
                .with_data_root("net.route")
                .with_binding(route_config.as_str()),
                ComponentDrawerDescriptor::new(
                    "net.ReplicationSchema",
                    "plugins://net/editor/replication_schema.zui",
                    "net.editor.replication_schema",
                )
                .with_template_id(NET_TEMPLATE_ID)
                .with_data_root("net.replication_schema")
                .with_binding(schema_open.as_str()),
            ],
            asset_creation_templates: vec![AssetCreationTemplateDescriptor::new(
                NET_REPLICATION_SCHEMA_TEMPLATE_ID,
                "Network Replication Schema",
                NET_REPLICATION_SCHEMA_ASSET_KIND,
                schema_create,
            )
            .with_default_document("plugins://net/editor/replication_schema.default.toml")
            .with_required_capabilities([NET_AUTHORING_CAPABILITY])],
            graph_editors: vec![GraphEditorDescriptor::new(
                NET_REPLICATION_SCHEMA_ASSET_KIND,
                NET_AUTHORING_VIEW_ID,
                "Network Replication Schema",
                schema_open,
                schema_validate,
            )
            .with_compile_operation(schema_compile)
            .with_required_capabilities([NET_AUTHORING_CAPABILITY])],
            graph_node_palettes: vec![GraphNodePaletteDescriptor::new(
                NET_REPLICATION_SCHEMA_PALETTE_ID,
                NET_REPLICATION_SCHEMA_ASSET_KIND,
            )
            .with_node(
                GraphNodeDescriptor::new("network_identity", "Network Identity", "Replication")
                    .with_output(GraphPinDescriptor::new("object", "net.object_id")),
            )
            .with_node(
                GraphNodeDescriptor::new(
                    "replicated_component",
                    "Replicated Component",
                    "Replication",
                )
                .with_input(GraphPinDescriptor::new("component", "reflect.type").required(true))
                .with_output(GraphPinDescriptor::new("schema", "net.replication_schema")),
            )
            .with_required_capabilities([NET_AUTHORING_CAPABILITY])],
            ..Default::default()
        },
    )
}

fn configure_operation(
    path: EditorOperationPath,
    display_name: &'static str,
    menu_path: &'static str,
    payload_schema_id: &'static str,
) -> EditorOperationDescriptor {
    EditorOperationDescriptor::new(path, display_name)
        .with_menu_path(menu_path)
        .with_payload_schema_id(payload_schema_id)
        .with_required_capabilities([NET_AUTHORING_CAPABILITY])
}

fn menu_item(path: &'static str, operation: EditorOperationPath) -> EditorMenuItemDescriptor {
    EditorMenuItemDescriptor::new(path, operation)
        .with_required_capabilities([NET_AUTHORING_CAPABILITY])
}

fn operation_path(path: &'static str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::Operation)
}
