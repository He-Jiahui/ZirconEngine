use super::super::*;
use crate::core::commands::{EditorCommandDescriptor, EditorCommandMenuPath};

#[test]
fn editor_runtime_folds_menu_capabilities_into_the_shared_command_descriptor() {
    use crate::core::commands::{CommandEvalCtx, WhenClause};
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_menu_capability_command_when");
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let capability = "editor.extension.weather_authoring";
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(EditorCommandDescriptor::operation(operation_path.clone()))
        .unwrap();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::for_operation(operation_path.clone())
                .with_required_capabilities([capability]),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("menu capability should fold into its extension-owned command");

    let descriptor = runtime
        .runtime
        .commands()
        .lock()
        .command(operation_path.as_str())
        .cloned()
        .expect("shared command descriptor");
    assert_eq!(
        descriptor.required_capabilities(),
        &[capability.to_string()]
    );
    assert_eq!(
        descriptor.effective_when(),
        WhenClause::Capability(capability.to_string())
    );
    assert!(!descriptor.is_enabled(&CommandEvalCtx::interactive()));
    assert!(descriptor.is_enabled(&CommandEvalCtx::interactive().with_capabilities([capability])));
}

#[test]
fn editor_runtime_consumes_plugin_command_descriptors_into_the_shared_registry() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_operation");
    let operation_path = EditorOperationPath::parse("weather.tools.reset_layout").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &operation_path,
                    "tools",
                    &["weather"],
                ))
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");
    {
        let shell = runtime.runtime.shell().lock();
        let stored = shell.contributions.snapshot();
        assert!(stored
            .commands(&crate::core::extension::CapabilitySet::default())
            .any(|command| command.id() == &operation_path));
    }
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
fn explicit_plugin_operation_keeps_its_identity_without_synthetic_history() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_operation_stack_identity");
    let operation_path = EditorOperationPath::parse("zzz.tools.reset_layout").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone())
                .with_menu_path(EditorCommandMenuPath::builtin(
                    &operation_path,
                    "tools",
                    &["zzz"],
                ))
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

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
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_menu_operation");
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone())
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::for_operation(operation_path).with_priority(10),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
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
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_menu_operation_arguments");
    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone())
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::for_operation(operation_path))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
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
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id: "External.OperationAudit".to_string(),
            after_delivery_cursor: 0,
            max_deliveries: 256,
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
        .register_editor_extension(extension.into_contribution_batch().unwrap())
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
        .register_editor_extension(extension.into_contribution_batch().unwrap())
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
