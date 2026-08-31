use std::sync::Arc;

use zircon_editor::core::commands::{
    EditorCommandDescriptor, EditorCommandMenuPath, EditorCommandRegistryError,
};
use zircon_editor::core::editing::operation::{
    EditOperationTarget, OperationCommandFactoryRegistration,
};
use zircon_editor::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_editor::core::editor_operation::EditorOperationPath;

use crate::capability::{NAVIGATION_AUTHORING_CAPABILITY, NAVIGATION_GIZMOS_CAPABILITY};
use zircon_runtime::core::framework::navigation::{
    NAVIGATION_BAKE_SCENE_OPERATION, NAVIGATION_BAKE_SURFACE_OPERATION,
    NAVIGATION_CLEAR_SURFACE_OPERATION,
};

use crate::extension_ids::{
    NAVIGATION_AGENTS_VIEW_ID, NAVIGATION_OPEN_SETTINGS_OPERATION,
    NAVIGATION_TOGGLE_GIZMOS_OPERATION,
};
use crate::operation_command::NavigationOperationCommandFactory;

pub(super) fn register(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    for spec in operation_specs() {
        let operation = parse_operation(spec.path)?;
        let command = EditorCommandDescriptor::operation(operation.clone())
            .with_menu_path(EditorCommandMenuPath::builtin(
                &operation,
                spec.menu_root,
                spec.menu_groups,
            ))
            .with_callable_from_remote(false)
            .with_required_capabilities([spec.capability]);
        let command = match spec.route {
            OperationRoute::Edit { payload_schema } => {
                command.with_payload_schema_id(payload_schema)
            }
            OperationRoute::OpenView(view_id) => command.with_event(EditorEvent::WorkbenchMenu(
                MenuAction::OpenView(ViewDescriptorId::new(view_id)),
            )),
            OperationRoute::ToggleOverlay => {
                command.with_payload_schema_id("navigation.overlay.toggle.v1")
            }
        };
        if matches!(spec.route, OperationRoute::Edit { .. }) {
            let factory =
                NavigationOperationCommandFactory::for_operation(&operation).map_err(|error| {
                    EditorExtensionRegistryError::Command(
                        EditorCommandRegistryError::OperationFactory(error),
                    )
                })?;
            registry.register_operation_command(
                command,
                OperationCommandFactoryRegistration::new(
                    operation.clone(),
                    spec.display_name,
                    EditOperationTarget::EditWorkspace,
                    Arc::new(factory),
                ),
            )?;
        } else {
            registry.register_command(command)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum OperationRoute {
    Edit { payload_schema: &'static str },
    OpenView(&'static str),
    ToggleOverlay,
}

#[derive(Clone, Copy)]
struct OperationSpec {
    path: &'static str,
    display_name: &'static str,
    menu_root: &'static str,
    menu_groups: &'static [&'static str],
    capability: &'static str,
    route: OperationRoute,
}

fn operation_specs() -> [OperationSpec; 5] {
    [
        OperationSpec {
            path: NAVIGATION_BAKE_SCENE_OPERATION,
            display_name: "Bake Navigation Scene",
            menu_root: "plugins",
            menu_groups: &["navigation"],
            capability: NAVIGATION_AUTHORING_CAPABILITY,
            route: OperationRoute::Edit {
                payload_schema: "navigation.bake.scene.v1",
            },
        },
        OperationSpec {
            path: NAVIGATION_BAKE_SURFACE_OPERATION,
            display_name: "Bake Selected NavMesh Surface",
            menu_root: "plugins",
            menu_groups: &["navigation"],
            capability: NAVIGATION_AUTHORING_CAPABILITY,
            route: OperationRoute::Edit {
                payload_schema: "navigation.bake.selected_surface.v1",
            },
        },
        OperationSpec {
            path: NAVIGATION_CLEAR_SURFACE_OPERATION,
            display_name: "Clear NavMesh Surface Bake",
            menu_root: "plugins",
            menu_groups: &["navigation"],
            capability: NAVIGATION_AUTHORING_CAPABILITY,
            route: OperationRoute::Edit {
                payload_schema: "navigation.bake.clear_surface.v1",
            },
        },
        OperationSpec {
            path: NAVIGATION_OPEN_SETTINGS_OPERATION,
            display_name: "Open Navigation Settings",
            menu_root: "plugins",
            menu_groups: &["navigation"],
            capability: NAVIGATION_AUTHORING_CAPABILITY,
            route: OperationRoute::OpenView(NAVIGATION_AGENTS_VIEW_ID),
        },
        OperationSpec {
            path: NAVIGATION_TOGGLE_GIZMOS_OPERATION,
            display_name: "Toggle Navigation Gizmos",
            menu_root: "view",
            menu_groups: &["debug_overlays"],
            capability: NAVIGATION_GIZMOS_CAPABILITY,
            route: OperationRoute::ToggleOverlay,
        },
    ]
}

fn parse_operation(path: &str) -> Result<EditorOperationPath, EditorExtensionRegistryError> {
    EditorOperationPath::parse(path).map_err(EditorExtensionRegistryError::OperationPath)
}
