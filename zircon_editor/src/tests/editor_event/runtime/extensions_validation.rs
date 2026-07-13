use super::*;
use crate::core::commands::EditorCommandDescriptor;

#[test]
fn editor_runtime_rejects_menu_capabilities_without_an_extension_owned_command() {
    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    };
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_menu_capability_without_owner");
    let operation_path = EditorOperationPath::parse("file.project.open").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new("Tools/Project/Open", operation_path.clone())
                .with_required_capabilities(["editor.extension.project_tools"]),
        )
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension)
        .unwrap_err();

    assert_eq!(
        error,
        EditorExtensionRegistryError::MenuCapabilitiesRequireContributedCommand {
            command_id: operation_path,
        }
    );
    assert!(runtime.runtime.shell().lock().editor_extensions.is_empty());
}

#[test]
fn editor_runtime_rejects_serde_retained_command_ids_as_menu_capability_owners_atomically() {
    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    };
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_menu_capability_retained_id");
    let operation_path = EditorOperationPath::parse("weather.retained.refresh").unwrap();
    let mut retained = EditorExtensionRegistry::default();
    retained
        .register_command(EditorCommandDescriptor::pending_operation(
            operation_path.clone(),
            "Refresh Retained Weather",
        ))
        .unwrap();
    assert_eq!(retained.take_command_contributions().len(), 1);
    assert!(retained.pending_commands().next().is_none());
    assert!(retained.command_ids().any(|id| id == &operation_path));

    let encoded = serde_json::to_value(&retained).unwrap();
    let mut extension: EditorExtensionRegistry = serde_json::from_value(encoded).unwrap();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new(
                "Tools/Weather/Refresh Retained Weather",
                operation_path.clone(),
            )
            .with_required_capabilities(["editor.extension.weather_authoring"]),
        )
        .unwrap();

    let command_ids_before = runtime
        .runtime
        .commands()
        .lock()
        .commands()
        .map(|command| command.id().clone())
        .collect::<Vec<_>>();
    let extension_count_before = runtime.runtime.shell().lock().editor_extensions.len();
    let descriptor_ids_before = runtime
        .runtime
        .descriptors()
        .into_iter()
        .map(|descriptor| descriptor.descriptor_id)
        .collect::<Vec<_>>();

    let error = runtime
        .runtime
        .register_editor_extension(extension)
        .unwrap_err();

    assert_eq!(
        error,
        EditorExtensionRegistryError::MenuCapabilitiesRequireContributedCommand {
            command_id: operation_path.clone(),
        }
    );
    assert_eq!(
        runtime
            .runtime
            .commands()
            .lock()
            .commands()
            .map(|command| command.id().clone())
            .collect::<Vec<_>>(),
        command_ids_before
    );
    assert!(runtime
        .runtime
        .commands()
        .lock()
        .command(operation_path.as_str())
        .is_none());
    assert_eq!(
        runtime.runtime.shell().lock().editor_extensions.len(),
        extension_count_before
    );
    assert_eq!(
        runtime
            .runtime
            .descriptors()
            .into_iter()
            .map(|descriptor| descriptor.descriptor_id)
            .collect::<Vec<_>>(),
        descriptor_ids_before
    );
}

#[test]
fn editor_runtime_rejects_menu_items_to_missing_operations() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_menu_missing_operation");
    let mut extension = EditorExtensionRegistry::default();
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path,
        ))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor command weather.cloud_layer.refresh is not registered"
    );
}

#[test]
fn editor_extension_registry_rejects_invalid_menu_item_paths() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    for path in [
        "",
        "Tools",
        "Tools//Refresh",
        "/Tools/Refresh",
        "Tools/Refresh/",
    ] {
        let mut extension = EditorExtensionRegistry::default();
        let error = extension
            .register_menu_item(EditorMenuItemDescriptor::new(path, operation_path.clone()))
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("editor menu item path `{path}` is invalid")
        );
    }
}

#[test]
fn editor_extension_registry_rejects_view_ids_that_cannot_form_open_operation_paths() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    for view_id in ["weather/cloud_layers", "weather.cloud layers"] {
        let mut extension = EditorExtensionRegistry::default();
        let error = extension
            .register_view(ViewDescriptor::new(view_id, "Cloud Layers", "Weather"))
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("editor operation path `view.{view_id}.open` is invalid")
        );
        assert!(extension.views().is_empty());
    }
}

#[test]
fn editor_runtime_rejects_duplicate_extension_view_without_registering_operations() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
    use crate::core::editor_operation::{EditorOperationControlRequest, EditorOperationPath};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_duplicate_extension_view");
    let mut first_extension = EditorExtensionRegistry::default();
    first_extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(first_extension)
        .expect("register first extension view");

    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut duplicate_extension = EditorExtensionRegistry::default();
    duplicate_extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers Duplicate",
            "Weather",
        ))
        .unwrap();
    duplicate_extension
        .register_command(EditorCommandDescriptor::pending_operation(
            operation_path.clone(),
            "Refresh Cloud Layers",
        ))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(duplicate_extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "view descriptor weather.cloud_layers already registered"
    );
    let operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    assert!(!operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array")
        .iter()
        .any(|operation| operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some(operation_path.as_str())));
}

#[test]
fn editor_runtime_rejects_generated_view_command_collision_atomically() {
    use crate::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_view_command_collision");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new("project", "Plugin Project", "Plugin"))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor command view.project.open already registered"
    );
    let retained_event = runtime
        .runtime
        .commands()
        .lock()
        .command("view.project.open")
        .and_then(EditorCommandDescriptor::event)
        .cloned();
    assert_eq!(
        retained_event.as_ref(),
        Some(&EditorEvent::WorkbenchMenu(MenuAction::OpenView(
            ViewDescriptorId::new("editor.project")
        )))
    );
    assert!(runtime.runtime.shell().lock().editor_extensions.is_empty());
    assert!(runtime
        .runtime
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "project"));
}

#[test]
fn editor_runtime_rejects_explicit_view_command_with_the_wrong_target_atomically() {
    use crate::core::editor_event::{EditorEvent, MenuAction};
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_view_command_wrong_target");
    let operation_path = EditorOperationPath::parse("view.weather.cloud_layers.open").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    extension
        .register_command(
            EditorCommandDescriptor::pending_operation(operation_path.clone(), "Open Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor command view.weather.cloud_layers.open does not open extension view weather.cloud_layers"
    );
    assert!(runtime
        .runtime
        .commands()
        .lock()
        .command(operation_path.as_str())
        .is_none());
    assert!(runtime.runtime.shell().lock().editor_extensions.is_empty());
    assert!(runtime
        .runtime
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "weather.cloud_layers"));
}

#[test]
fn editor_runtime_accepts_explicit_view_command_only_for_the_matching_target() {
    use crate::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_view_command_matching_target");
    let view_id = "weather.cloud_layers";
    let operation_path = EditorOperationPath::parse("view.weather.cloud_layers.open").unwrap();
    let expected_event =
        EditorEvent::WorkbenchMenu(MenuAction::OpenView(ViewDescriptorId::new(view_id)));
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(view_id, "Cloud Layers", "Weather"))
        .unwrap();
    extension
        .register_command(
            EditorCommandDescriptor::pending_operation(operation_path.clone(), "Open Cloud Layers")
                .with_event(expected_event.clone()),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register matching explicit view command");

    let registered_event = runtime
        .runtime
        .commands()
        .lock()
        .command(operation_path.as_str())
        .and_then(EditorCommandDescriptor::event)
        .cloned();
    assert_eq!(registered_event, Some(expected_event));
    let shell = runtime.runtime.shell().lock();
    let stored = shell
        .editor_extensions
        .last()
        .expect("stored extension registration")
        .registry();
    assert!(stored.pending_commands().next().is_none());
    assert!(stored.command_ids().any(|id| id == &operation_path));
}

#[test]
fn editor_runtime_rejects_duplicate_extension_menu_paths_without_registering_operations() {
    use crate::core::editor_event::{EditorEvent, MenuAction};
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::{EditorOperationControlRequest, EditorOperationPath};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_duplicate_extension_menu");
    let first_operation = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut first_extension = EditorExtensionRegistry::default();
    first_extension
        .register_command(
            EditorCommandDescriptor::pending_operation(
                first_operation.clone(),
                "Refresh Cloud Layers",
            )
            .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    first_extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            first_operation,
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(first_extension)
        .expect("register first extension menu");

    let second_operation = EditorOperationPath::parse("weather.cloud_layer.reset").unwrap();
    let mut duplicate_extension = EditorExtensionRegistry::default();
    duplicate_extension
        .register_command(EditorCommandDescriptor::pending_operation(
            second_operation.clone(),
            "Reset Cloud Layers",
        ))
        .unwrap();
    duplicate_extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            second_operation.clone(),
        ))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(duplicate_extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor menu item Tools/Weather/Refresh Cloud Layers already registered"
    );
    let operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    assert!(!operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array")
        .iter()
        .any(|operation| operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some(second_operation.as_str())));
}

#[test]
fn editor_runtime_rejects_component_drawer_bindings_to_missing_operations() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_component_drawer_missing_binding");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                "weather.Component.CloudLayer",
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_binding("weather.cloud_layer.refresh"),
        )
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor command weather.cloud_layer.refresh is not registered"
    );
}

#[test]
fn editor_extension_registry_rejects_invalid_component_drawer_operation_bindings() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};

    let mut extension = EditorExtensionRegistry::default();
    let error = extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                "weather.Component.CloudLayer",
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_binding("weather.refresh"),
        )
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor operation path `weather.refresh` is invalid"
    );
}

#[test]
fn editor_extension_registry_rejects_invalid_component_drawer_template_metadata() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};

    let mut extension = EditorExtensionRegistry::default();
    let error = extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                "weather.Component.CloudLayer",
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_template_id(" weather.cloud_layer.inspector"),
        )
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor component drawer template id ` weather.cloud_layer.inspector` is invalid"
    );
}

#[test]
fn editor_extension_registry_rejects_non_zui_ui_template_documents() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorUiTemplateDescriptor};

    let mut extension = EditorExtensionRegistry::default();
    let error = extension
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "weather.cloud_layer.inspector",
            "asset://weather/editor/cloud_layer.inspector.toml",
        ))
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor ui template document `asset://weather/editor/cloud_layer.inspector.toml` must reference a supported editor UI asset"
    );
}

#[test]
fn editor_extension_registry_rejects_non_zui_component_drawer_documents() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};

    let mut extension = EditorExtensionRegistry::default();
    let error = extension
        .register_component_drawer(ComponentDrawerDescriptor::new(
            "weather.Component.CloudLayer",
            "asset://weather/editor/cloud_layer.inspector.toml",
            "weather.editor.CloudLayerInspectorController",
        ))
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor component drawer document `asset://weather/editor/cloud_layer.inspector.toml` must reference a supported editor UI asset"
    );
}
