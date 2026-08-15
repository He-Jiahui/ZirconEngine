use super::super::*;
use crate::core::commands::EditorCommandDescriptor;
#[test]
fn editor_runtime_consumes_plugin_registration_reports_with_capability_gate() {
    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorMenuItemDescriptor, ViewDescriptor,
    };
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    };
    use crate::core::extension::InspectorCustomizationDescriptor;
    use crate::core::plugin::EditorPluginRegistrationReport;
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use crate::ui::host::EditorManager;
    use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
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
            "plugin.weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    let operation_path = EditorOperationPath::parse("plugin.weather.cloud_layer.refresh").unwrap();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path.clone(),
        ))
        .unwrap();
    let component_type = "plugin.weather.CloudLayer";
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
                component_type,
                "plugins://weather/editor/cloud_layer.inspector.zui",
                "plugin.weather.CloudLayerInspectorController",
            )
            .with_id("plugin.weather.cloud_layer")
            .with_binding("plugin.weather.cloud_layer.refresh"),
        )
        .unwrap();
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

    runtime
        .runtime
        .register_editor_plugin_registration(EditorPluginRegistrationReport {
            package_manifest: PluginPackageManifest::new("weather", "Weather").with_editor_module(
                PluginModuleManifest::editor("weather.editor", "zircon_plugin_weather_editor")
                    .with_capabilities([capability.clone()]),
            ),
            capabilities: vec![capability.clone()],
            extensions: extension,
            lifecycle: crate::core::plugin::sdk::lifecycle::EditorPluginLifecycleReport::default(),
            successful_lifecycle_stages: Vec::new(),
            failed_lifecycle_stages: Vec::new(),
            runtime_event_consumers:
                crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry::default(),
            diagnostics: Vec::new(),
        })
        .expect("register editor plugin report");
    runtime.runtime.refresh_reflection();

    let disabled_component = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("plugin component snapshot while disabled");
    assert!(!disabled_component.customization_available);
    assert_eq!(disabled_component.customization_ui_document, None);

    assert!(runtime
        .runtime
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "plugin.weather.cloud_layers"));
    let disabled_menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new(
                "editor/workbench/menu/view/view.plugin.weather.cloud_layers.open",
            ),
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
            == Some("plugin.weather.cloud_layer.refresh")));
    let disabled_invoke = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path.clone(),
        )),
    );
    assert_eq!(
        disabled_invoke.error.as_deref(),
        Some(
            "editor command plugin.weather.cloud_layer.refresh requires disabled capabilities: editor.extension.weather_authoring"
        )
    );

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .set_editor_capabilities_enabled(&[capability.clone()], true)
        .unwrap();
    runtime.runtime.refresh_reflection();

    let enabled_component = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("plugin component snapshot while enabled");
    assert!(enabled_component.customization_available);
    assert_eq!(
        enabled_component.customization_ui_document.as_deref(),
        Some("plugins://weather/editor/cloud_layer.inspector.zui")
    );
    assert_eq!(
        enabled_component.customization_controller.as_deref(),
        Some("plugin.weather.CloudLayerInspectorController")
    );

    let descriptor = runtime
        .runtime
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "plugin.weather.cloud_layers")
        .expect("enabled plugin view descriptor registered");
    assert_eq!(
        descriptor.required_capabilities,
        vec!["editor.extension.weather_authoring"]
    );
    let enabled_menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new(
                "editor/workbench/menu/view/view.plugin.weather.cloud_layers.open",
            ),
        });
    assert!(matches!(
        enabled_menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Cloud Layers"
                && node.properties["operation_path"].reflected_value
                    == json!("view.plugin.weather.cloud_layers.open")
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
                == Some("plugin.weather.cloud_layer.refresh")
        })
        .expect("weather operation is discoverable when capability is enabled");
    assert_eq!(
        weather_operation.get("required_capabilities"),
        Some(&json!(["editor.extension.weather_authoring"]))
    );
    assert!(enabled_operations.iter().any(|operation| {
        operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some("plugin.weather.cloud_layer.refresh")
    }));
    let enabled_invoke = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path,
        )),
    );
    assert!(enabled_invoke.error.is_none());

    manager
        .set_editor_capabilities_enabled(&[capability], false)
        .unwrap();
    runtime.runtime.refresh_reflection();
    let disabled_again_component = runtime
        .runtime
        .editor_snapshot()
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("plugin component snapshot after capability revocation");
    assert!(!disabled_again_component.customization_available);
    assert_eq!(disabled_again_component.customization_ui_document, None);
}

#[test]
fn editor_runtime_snapshots_enabled_plugin_templates_by_owner_and_capability() {
    use std::{collections::BTreeMap, sync::Arc};

    use crate::core::asset::AssetTypeRegistry;
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorUiTemplateDescriptor};
    use crate::core::plugin::EditorPluginRegistrationReport;
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use crate::ui::host::EditorManager;
    use zircon_runtime::{plugin::PluginModuleManifest, plugin::PluginPackageManifest};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_plugin_template_snapshot",
        &[],
    );
    let capability = "editor.extension.weather_authoring".to_string();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "plugin.weather.cloud_layer.inspector",
            "plugins://weather/editor/cloud_layer.inspector.zui",
        ))
        .expect("template descriptor should be accepted");

    runtime
        .runtime
        .register_editor_plugin_registration(EditorPluginRegistrationReport {
            package_manifest: PluginPackageManifest::new("weather", "Weather").with_editor_module(
                PluginModuleManifest::editor("weather.editor", "zircon_plugin_weather_editor")
                    .with_capabilities([capability.clone()]),
            ),
            capabilities: vec![capability.clone()],
            extensions: extension,
            lifecycle: crate::core::plugin::sdk::lifecycle::EditorPluginLifecycleReport::default(),
            successful_lifecycle_stages: Vec::new(),
            failed_lifecycle_stages: Vec::new(),
            runtime_event_consumers:
                crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry::default(),
            diagnostics: Vec::new(),
        })
        .expect("register plugin template descriptor");

    let (registered_generation, disabled_capabilities, disabled_templates) =
        runtime.runtime.enabled_plugin_template_descriptors();
    let (disabled_revision, disabled_revision_capabilities) =
        runtime.runtime.plugin_template_revision();
    assert!(registered_generation > 0);
    assert_eq!(disabled_revision, registered_generation);
    assert!(!disabled_capabilities.contains(&capability));
    assert_eq!(disabled_revision_capabilities, disabled_capabilities);
    assert!(!disabled_templates.contains_key("weather"));

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager should be available");
    manager
        .set_editor_capabilities_enabled(&[capability.clone()], true)
        .expect("enable plugin template capability");

    let (enabled_generation, enabled_capabilities, enabled_templates) =
        runtime.runtime.enabled_plugin_template_descriptors();
    let (enabled_revision, enabled_revision_capabilities) =
        runtime.runtime.plugin_template_revision();
    assert_eq!(enabled_generation, registered_generation);
    assert_eq!(enabled_revision, registered_generation);
    assert!(enabled_capabilities.contains(&capability));
    assert_eq!(enabled_revision_capabilities, enabled_capabilities);
    assert_eq!(
        enabled_templates
            .get("weather")
            .expect("enabled plugin owner should expose templates")
            .iter()
            .map(|descriptor| descriptor.id())
            .collect::<Vec<_>>(),
        vec!["plugin.weather.cloud_layer.inspector"]
    );

    let unknown_owner_error = runtime
        .runtime
        .replace_editor_plugin_ui_template_contributions(
            "unknown.weather",
            std::iter::empty::<EditorUiTemplateDescriptor>(),
            BTreeMap::new(),
        )
        .expect_err("template replacement must not register an unknown extension owner");
    assert!(matches!(
        unknown_owner_error,
        crate::core::editor_extension::EditorExtensionRegistryError::UnknownExtensionOwner {
            ref owner_id
        } if owner_id == "unknown.weather"
    ));
    assert_eq!(
        runtime.runtime.plugin_template_revision().0,
        enabled_generation,
        "rejected replacement must not advance the template generation"
    );

    {
        let mut shell = runtime.runtime.shell().lock();
        shell
            .asset_type_registry_cache
            .store(Vec::new(), Arc::new(AssetTypeRegistry::default()));
    }

    runtime
        .runtime
        .replace_editor_plugin_ui_template_contributions(
            "weather",
            [EditorUiTemplateDescriptor::new(
                "plugin.weather.cloud_layer.inspector",
                "plugins://weather/editor/cloud_layer.inspector.reloaded.zui",
            )],
            BTreeMap::new(),
        )
        .expect("registered plugin templates should support an atomic replacement");

    let asset_cache_counts = {
        let mut shell = runtime.runtime.shell().lock();
        assert!(
            shell.asset_type_registry_cache.get(&[]).is_some(),
            "template replacement must not invalidate the unrelated asset-type cache"
        );
        shell.asset_type_registry_cache.counts()
    };
    assert_eq!(asset_cache_counts, (1, 1));

    let (reloaded_generation, _, reloaded_templates) =
        runtime.runtime.enabled_plugin_template_descriptors();
    assert!(reloaded_generation > enabled_generation);
    assert_eq!(
        reloaded_templates
            .get("weather")
            .and_then(|templates| templates.first())
            .map(|descriptor| descriptor.ui_document()),
        Some("plugins://weather/editor/cloud_layer.inspector.reloaded.zui")
    );

    manager
        .set_editor_capabilities_enabled(&[capability.clone()], false)
        .expect("disable plugin template capability after reload");
    assert!(!runtime
        .runtime
        .enabled_plugin_template_descriptors()
        .2
        .contains_key("weather"));

    manager
        .set_editor_capabilities_enabled(&[capability], true)
        .expect("re-enable plugin template capability after reload");
    assert_eq!(
        runtime
            .runtime
            .enabled_plugin_template_descriptors()
            .2
            .get("weather")
            .and_then(|templates| templates.first())
            .map(|descriptor| descriptor.ui_document()),
        Some("plugins://weather/editor/cloud_layer.inspector.reloaded.zui")
    );
}

#[test]
fn editor_runtime_exposes_plugin_inspector_customization_surface_for_inspector_lookup() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorUiTemplateDescriptor};
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::extension::{InspectorCustomization, InspectorCustomizationDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_inspector_customization");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(EditorCommandDescriptor::operation(
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
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
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
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register editor extension");

    let customization = runtime
        .runtime
        .inspector_customization("weather.Component.CloudLayer")
        .expect("inspector customization registered");
    let surface = customization.surface().expect("customization UI surface");
    assert_eq!(
        surface.ui_document(),
        "asset://weather/editor/cloud_layer.inspector.zui"
    );
    assert_eq!(
        surface.controller(),
        "weather.editor.CloudLayerInspectorController"
    );
    assert_eq!(surface.template_id(), Some("weather.cloud_layer.inspector"));
    assert_eq!(
        surface.data_root(),
        Some("inspector.plugin_components.weather.Component.CloudLayer")
    );
    assert_eq!(surface.bindings(), ["weather.cloud_layer.refresh"]);

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
fn editor_snapshot_resolves_enabled_inspector_customization_for_selected_dynamic_component() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::extension::InspectorCustomizationDescriptor;
    use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_inspector_customization_snapshot");
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
        .register_command(EditorCommandDescriptor::operation(
            operation_path,
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
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
        .register_editor_extension(extension.into_contribution_batch().unwrap())
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

    assert!(component.customization_available);
    assert_eq!(
        component.customization_ui_document.as_deref(),
        Some("asset://weather/editor/cloud_layer.inspector.zui")
    );
    assert_eq!(
        component.customization_controller.as_deref(),
        Some("weather.editor.CloudLayerInspectorController")
    );
    assert_eq!(
        component.customization_template_id.as_deref(),
        Some("weather.cloud_layer.inspector")
    );
    assert_eq!(
        component.customization_bindings,
        ["weather.cloud_layer.refresh"]
    );
    assert_eq!(component.diagnostic, None);
    assert_eq!(
        component.properties[0].field_id,
        "weather.Component.CloudLayer.coverage"
    );
}

#[test]
fn editor_snapshot_resolves_plugin_field_editors_from_active_contributions() {
    use crate::core::extension::{
        ContributionBatch, FieldEditorDefinition, FieldEditorInstance, FieldEditorKind,
    };
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use crate::ui::host::EditorManager;
    use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_plugin_field_editor_snapshot",
        &[],
    );
    let capability = "editor.extension.weather_authoring".to_string();
    let component_type = "plugin.weather.CloudLayer";
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
                        .with_property("coverage", "plugin.weather.CloudCoverage", true),
                )
                .unwrap();
            scene
                .set_dynamic_component(selected_node, component_type, json!({ "coverage": 0.75 }))
                .unwrap();
        });
    }

    let mut batch = ContributionBatch::default().with_required_capabilities([capability.clone()]);
    batch
        .register_field_editor(FieldEditorDefinition::new(
            "plugin.weather.CloudCoverage",
            |_| FieldEditorInstance::new(FieldEditorKind::Color),
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(batch)
        .expect("register field editor contribution");

    let disabled_snapshot = runtime.runtime.editor_snapshot();
    let disabled = disabled_snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");
    assert_eq!(
        disabled.properties[0].field_editor.kind(),
        FieldEditorKind::Auto
    );

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .set_editor_capabilities_enabled(&[capability], true)
        .expect("enable field editor capability");
    assert_eq!(
        disabled.properties[0].field_editor.kind(),
        FieldEditorKind::Auto,
        "published editor snapshots retain resolved field metadata after capability changes"
    );
    let enabled_snapshot = runtime.runtime.editor_snapshot();
    let enabled = enabled_snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");
    assert_eq!(
        enabled.properties[0].field_editor.kind(),
        FieldEditorKind::Color
    );
    manager
        .set_editor_capabilities_enabled(&[capability], false)
        .expect("disable field editor capability");
    assert_eq!(
        enabled.properties[0].field_editor.kind(),
        FieldEditorKind::Color,
        "published snapshots must retain resolved field metadata after capability removal"
    );
    let revoked_snapshot = runtime.runtime.editor_snapshot();
    let revoked = revoked_snapshot
        .inspector
        .as_ref()
        .expect("inspector")
        .plugin_components
        .iter()
        .find(|component| component.component_id == component_type)
        .expect("dynamic component snapshot");
    assert_eq!(
        revoked.properties[0].field_editor.kind(),
        FieldEditorKind::Auto,
        "new snapshots must fall back when the field editor contribution is no longer active"
    );
}

#[test]
fn editor_snapshot_hides_inspector_customization_when_extension_capability_is_disabled() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::extension::InspectorCustomizationDescriptor;
    use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_inspector_customization_disabled");
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
        .register_command(EditorCommandDescriptor::operation(
            operation_path,
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
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
            extension.into_contribution_batch().unwrap(),
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

    assert!(!component.customization_available);
    assert_eq!(component.customization_ui_document, None);
    assert_eq!(component.customization_controller, None);
    assert!(component.diagnostic.as_deref().is_some_and(|diagnostic| {
        diagnostic.contains("enabled editor extension registers a customization")
    }));
}
