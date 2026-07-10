use super::*;

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
        "editor operation weather.cloud_layer.refresh is not registered"
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
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationDescriptor, EditorOperationPath,
    };

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
        .register_operation(EditorOperationDescriptor::new(
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
fn editor_runtime_rejects_duplicate_extension_menu_paths_without_registering_operations() {
    use crate::core::editor_event::{EditorEvent, MenuAction};
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationDescriptor, EditorOperationPath,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_duplicate_extension_menu");
    let first_operation = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut first_extension = EditorExtensionRegistry::default();
    first_extension
        .register_operation(
            EditorOperationDescriptor::new(first_operation.clone(), "Refresh Cloud Layers")
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
        .register_operation(EditorOperationDescriptor::new(
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
        "editor operation weather.cloud_layer.refresh is not registered"
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
