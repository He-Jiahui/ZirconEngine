use super::*;

#[test]
fn editor_runtime_accepts_plugin_extension_operations_for_later_invocation() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::{
        EditorOperationDescriptor, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_operation");
    let operation_path = EditorOperationPath::parse("weather.tools.reset_layout").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Weather Reset Layout")
                .with_menu_path("Tools/Weather/Reset Layout")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    let record = runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

    assert_eq!(
        record.operation_id.as_deref(),
        Some("weather.tools.reset_layout")
    );
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("weather.tools.reset_layout")
    );
}

#[test]
fn explicit_plugin_operation_records_its_own_undo_stack_entry_when_reusing_builtin_event() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::{
        EditorOperationDescriptor, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource, UndoableEditorOperation,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_operation_stack_identity");
    let operation_path = EditorOperationPath::parse("zzz.tools.reset_layout").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Plugin Reset Layout")
                .with_menu_path("Tools/Zzz/Reset Layout")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout))
                .with_undoable(UndoableEditorOperation::new("Plugin Reset Layout")),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

    let stack = runtime.runtime.operation_stack();
    assert_eq!(stack.undo_stack().len(), 1);
    assert_eq!(
        stack.undo_stack()[0].operation_id.as_str(),
        "zzz.tools.reset_layout"
    );
    assert_eq!(stack.undo_stack()[0].display_name, "Plugin Reset Layout");
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("zzz.tools.reset_layout")
    );
}

#[test]
fn editor_runtime_projects_plugin_menu_operations_into_remote_callable_reflection() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_menu_operation");
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new("Tools/Weather/Refresh Cloud Layers", operation_path)
                .with_priority(10)
                .with_shortcut("Ctrl+Alt+R"),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime.runtime.refresh_reflection();

    let menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/tools/weather.cloud_layer.refresh"),
        });
    assert!(matches!(
        menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Refresh Cloud Layers"
                && node.actions["workbench.menu.item.click"].binding_symbol == "EditorOperation"
                && node.actions["workbench.menu.item.click"].callable_from_remote
                && node.properties["operation_path"].reflected_value
                    == json!("weather.cloud_layer.refresh")
                && node.properties["shortcut"].reflected_value == json!("Ctrl+Alt+R")
    ));

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/tools/weather.cloud_layer.refresh"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result)
            if result.error.is_none()
                && result.binding
                    .as_ref()
                    .and_then(|binding| binding.action.as_ref())
                    .map(|call| call.symbol.as_str())
                    == Some("EditorOperation")
    ));
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("weather.cloud_layer.refresh")
    );
}

#[test]
fn editor_operation_ui_binding_arguments_are_preserved_in_journal() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_menu_operation_arguments");
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path,
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: "External.OperationAudit".to_string(),
            display_name: "Operation Audit".to_string(),
        },
    );
    runtime.runtime.refresh_reflection();

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/tools/weather.cloud_layer.refresh"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: vec![
                UiBindingValue::String("storm".to_string()),
                UiBindingValue::Unsigned(7),
                UiBindingValue::Bool(true),
            ],
        });

    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result)
            if result.error.is_none()
                && result.binding
                    .as_ref()
                    .and_then(|binding| binding.action.as_ref())
                    .map(|call| call.arguments.len())
                    == Some(4)
    ));
    let journal = runtime.runtime.journal();
    let record = &journal.records()[0];
    assert_eq!(
        record.operation_id.as_deref(),
        Some("weather.cloud_layer.refresh")
    );
    assert_eq!(
        record.operation_arguments.as_ref(),
        Some(&json!(["storm", 7, true]))
    );
    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries {
            listener_id: "External.OperationAudit".to_string(),
        },
    );
    assert_eq!(
        deliveries.value["deliveries"][0]["operation_arguments"],
        json!(["storm", 7, true])
    );
}

#[test]
fn editor_runtime_registers_plugin_views_as_activity_descriptors() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_view_descriptor");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime.runtime.refresh_reflection();

    let descriptor = runtime
        .runtime
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "weather.cloud_layers")
        .expect("plugin view descriptor registered");
    assert_eq!(descriptor.default_title, "Cloud Layers");
    assert_eq!(descriptor.icon_key, "weather.cloud_layers");
    assert!(runtime
        .runtime
        .activity_view_descriptor("weather.cloud_layers")
        .is_some());
}

#[test]
fn editor_runtime_projects_plugin_views_into_view_menu_operations() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_view_menu_operation");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime.runtime.refresh_reflection();

    let menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/view/view.weather.cloud_layers.open"),
        });
    assert!(matches!(
        menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Cloud Layers"
                && node.properties["operation_path"].reflected_value
                    == json!("view.weather.cloud_layers.open")
                && node.actions["workbench.menu.item.click"].binding_symbol == "EditorOperation"
                && node.actions["workbench.menu.item.click"].callable_from_remote
    ));

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/view/view.weather.cloud_layers.open"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result) if result.error.is_none()
    ));
    assert!(runtime
        .runtime
        .current_view_instances()
        .iter()
        .any(|instance| instance.descriptor_id.0 == "weather.cloud_layers"));
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("view.weather.cloud_layers.open")
    );
}

#[test]
fn editor_runtime_consumes_plugin_registration_reports_with_capability_gate() {
    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorMenuItemDescriptor, ViewDescriptor,
    };
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationDescriptor, EditorOperationInvocation,
        EditorOperationPath,
    };
    use crate::core::editor_plugin::EditorPluginRegistrationReport;
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use crate::ui::host::EditorManager;
    use zircon_runtime::{plugin::PluginModuleManifest, plugin::PluginPackageManifest};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_plugin_registration_gate",
        &[],
    );
    let capability = "editor.extension.weather_authoring".to_string();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path.clone(),
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_plugin_registration(EditorPluginRegistrationReport {
            package_manifest: PluginPackageManifest::new("weather", "Weather").with_editor_module(
                PluginModuleManifest::editor("weather.editor", "zircon_plugin_weather_editor")
                    .with_capabilities([capability.clone()]),
            ),
            capabilities: vec![capability.clone()],
            extensions: extension,
            lifecycle:
                crate::core::editor_plugin_sdk::lifecycle::EditorPluginLifecycleReport::default(),
            diagnostics: Vec::new(),
        })
        .expect("register editor plugin report");
    runtime.runtime.refresh_reflection();

    assert!(runtime
        .runtime
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "weather.cloud_layers"));
    let disabled_menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/view/view.weather.cloud_layers.open"),
        });
    assert!(matches!(disabled_menu, UiControlResponse::Node(None)));
    let disabled_operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    assert!(!disabled_operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array")
        .iter()
        .any(|operation| operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some("weather.cloud_layer.refresh")));
    let disabled_invoke = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path.clone(),
        )),
    );
    assert_eq!(
        disabled_invoke.error.as_deref(),
        Some(
            "editor operation weather.cloud_layer.refresh requires disabled capabilities: editor.extension.weather_authoring"
        )
    );

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .set_editor_capabilities_enabled(&[capability], true)
        .unwrap();
    runtime.runtime.refresh_reflection();

    let descriptor = runtime
        .runtime
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "weather.cloud_layers")
        .expect("enabled plugin view descriptor registered");
    assert_eq!(
        descriptor.required_capabilities,
        vec!["editor.extension.weather_authoring"]
    );
    let enabled_menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/view/view.weather.cloud_layers.open"),
        });
    assert!(matches!(
        enabled_menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Cloud Layers"
                && node.properties["operation_path"].reflected_value
                    == json!("view.weather.cloud_layers.open")
    ));
    let enabled_operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    let enabled_operations = enabled_operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array");
    let weather_operation = enabled_operations
        .iter()
        .find(|operation| {
            operation
                .get("operation_id")
                .and_then(serde_json::Value::as_str)
                == Some("weather.cloud_layer.refresh")
        })
        .expect("weather operation is discoverable when capability is enabled");
    assert_eq!(
        weather_operation.get("required_capabilities"),
        Some(&json!(["editor.extension.weather_authoring"]))
    );
    assert!(enabled_operations.iter().any(|operation| operation
        .get("operation_id")
        .and_then(serde_json::Value::as_str)
        == Some("weather.cloud_layer.refresh")));
    let enabled_invoke = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path,
        )),
    );
    assert!(enabled_invoke.error.is_none());
}

#[test]
fn editor_runtime_exposes_plugin_component_drawer_templates_for_inspector_lookup() {
    use crate::core::editor_extension::{
        ComponentDrawerDescriptor, EditorExtensionRegistry, EditorUiTemplateDescriptor,
    };
    use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_component_drawer");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(EditorOperationDescriptor::new(
            EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap(),
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "weather.cloud_layer.inspector",
            "asset://weather/editor/cloud_layer.inspector.zui",
        ))
        .unwrap();
    extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                "weather.Component.CloudLayer",
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_template_id("weather.cloud_layer.inspector")
            .with_data_root("inspector.plugin_components.weather.Component.CloudLayer")
            .with_binding("weather.cloud_layer.refresh"),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");

    let drawer = runtime
        .runtime
        .component_drawer_descriptor("weather.Component.CloudLayer")
        .expect("component drawer registered");
    assert_eq!(
        drawer.ui_document(),
        "asset://weather/editor/cloud_layer.inspector.zui"
    );
    assert_eq!(
        drawer.controller(),
        "weather.editor.CloudLayerInspectorController"
    );
    assert_eq!(drawer.template_id(), Some("weather.cloud_layer.inspector"));
    assert_eq!(
        drawer.data_root(),
        Some("inspector.plugin_components.weather.Component.CloudLayer")
    );
    assert_eq!(drawer.bindings(), ["weather.cloud_layer.refresh"]);

    let template = runtime
        .runtime
        .ui_template_descriptor("weather.cloud_layer.inspector")
        .expect("ui template registered");
    assert_eq!(
        template.ui_document(),
        "asset://weather/editor/cloud_layer.inspector.zui"
    );
}

#[test]
fn editor_snapshot_resolves_enabled_component_drawer_for_selected_dynamic_component() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};
    use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
    use zircon_runtime::plugin::ComponentTypeDescriptor;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_component_drawer_snapshot");
    let component_type = "weather.Component.CloudLayer";
    let selected_node = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("default selection")
        .id;

    {
        let shell = runtime.runtime.shell().lock();
        shell.state.world.with_world_mut(|scene| {
            scene
                .register_component_type(
                    ComponentTypeDescriptor::new(component_type, "weather", "Cloud Layer")
                        .with_property("coverage", "scalar", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected_node, component_type, json!({ "coverage": 0.75 }))
                .unwrap();
        });
    }

    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(EditorOperationDescriptor::new(
            operation_path,
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                component_type,
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_template_id("weather.cloud_layer.inspector")
            .with_data_root("inspector.plugin_components.weather.Component.CloudLayer")
            .with_binding("weather.cloud_layer.refresh"),
        )
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");

    let snapshot = runtime.runtime.editor_snapshot();
    let component = snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");

    assert!(component.drawer_available);
    assert_eq!(
        component.drawer_ui_document.as_deref(),
        Some("asset://weather/editor/cloud_layer.inspector.zui")
    );
    assert_eq!(
        component.drawer_controller.as_deref(),
        Some("weather.editor.CloudLayerInspectorController")
    );
    assert_eq!(
        component.drawer_template_id.as_deref(),
        Some("weather.cloud_layer.inspector")
    );
    assert_eq!(component.drawer_bindings, ["weather.cloud_layer.refresh"]);
    assert_eq!(component.diagnostic, None);
    assert_eq!(
        component.properties[0].field_id,
        "weather.Component.CloudLayer.coverage"
    );
}

#[test]
fn editor_snapshot_hides_component_drawer_when_extension_capability_is_disabled() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};
    use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};
    use zircon_runtime::plugin::ComponentTypeDescriptor;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_component_drawer_disabled");
    let component_type = "weather.Component.CloudLayer";
    let selected_node = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("default selection")
        .id;

    {
        let shell = runtime.runtime.shell().lock();
        shell.state.world.with_world_mut(|scene| {
            scene
                .register_component_type(
                    ComponentTypeDescriptor::new(component_type, "weather", "Cloud Layer")
                        .with_property("coverage", "scalar", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected_node, component_type, json!({ "coverage": 0.75 }))
                .unwrap();
        });
    }

    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(EditorOperationDescriptor::new(
            operation_path,
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                component_type,
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_binding("weather.cloud_layer.refresh"),
        )
        .unwrap();
    runtime
        .runtime
        .register_editor_extension_with_required_capabilities(
            extension,
            vec!["editor.extension.weather_authoring".to_string()],
        )
        .expect("register disabled extension");

    let snapshot = runtime.runtime.editor_snapshot();
    let component = snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");

    assert!(!component.drawer_available);
    assert_eq!(component.drawer_ui_document, None);
    assert_eq!(component.drawer_controller, None);
    assert!(component.diagnostic.as_deref().is_some_and(
        |diagnostic| diagnostic.contains("enabled editor extension registers a drawer")
    ));
}
