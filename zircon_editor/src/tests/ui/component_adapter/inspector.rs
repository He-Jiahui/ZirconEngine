use super::*;

#[test]
fn inspector_component_adapter_value_changed_updates_selected_name_draft() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_name_draft");
    let envelope = inspector_value_envelope("name", UiValue::String("Adapter Cube".to_string()));

    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap();

    assert!(result.changed);
    assert!(result.dirty);
    assert!(result.refresh_projection);
    assert_eq!(result.transaction_id.as_deref(), Some("inspector:name"));
    assert_eq!(result.mutation_source.as_deref(), Some("inspector"));
    assert_eq!(result.patches[0].control_id, "name");
    assert_eq!(
        result.patches[0].attributes.get("value"),
        Some(&UiValue::String("Adapter Cube".to_string()))
    );
    assert_eq!(
        result.patches[0].state_values.get("name"),
        Some(&UiValue::String("Adapter Cube".to_string()))
    );
    assert_eq!(
        harness
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Adapter Cube")
    );
}

#[test]
fn inspector_component_adapter_commit_updates_transform_draft() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_transform_commit");
    let envelope = inspector_commit_envelope("transform.translation.x", UiValue::Float(42.0));

    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap();

    assert!(result.changed);
    assert!(result.refresh_projection);
    assert_eq!(
        result.transaction_id.as_deref(),
        Some("inspector:transform.translation.x")
    );
    assert_eq!(result.mutation_source.as_deref(), Some("inspector"));
    assert_eq!(
        result.patches[0]
            .state_values
            .get("transform.translation.x"),
        Some(&UiValue::Float(42.0))
    );
    assert_eq!(
        harness
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.translation[0].as_str()),
        Some("42")
    );
}

#[test]
fn reflection_component_adapter_updates_selected_entity_name_draft() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_reflection_name");
    let envelope =
        reflection_commit_envelope("name", UiValue::String("Reflected Cube".to_string()));

    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap();

    assert!(result.changed);
    assert_eq!(result.transaction_id.as_deref(), Some("reflection:name"));
    assert_eq!(result.mutation_source.as_deref(), Some("reflection"));
    assert_eq!(
        harness
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Reflected Cube")
    );
}

#[test]
fn reflection_component_adapter_updates_selected_entity_translation_vector() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_reflection_translation");
    let envelope =
        reflection_commit_envelope("transform.translation", UiValue::Vec3([1.0, 2.0, 3.0]));

    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap();

    assert!(result.changed);
    assert_eq!(
        result.transaction_id.as_deref(),
        Some("reflection:transform.translation")
    );
    assert_eq!(result.mutation_source.as_deref(), Some("reflection"));
    assert_eq!(
        result.patches[0].state_values.get("transform.translation"),
        Some(&UiValue::Vec3([1.0, 2.0, 3.0]))
    );
}

#[test]
fn inspector_customization_adapter_invokes_only_enabled_declared_operation_bindings() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::extension::InspectorCustomizationDescriptor;

    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_inspector_customization_adapter_operation");
    let component_type = "weather.Component.CloudLayer";
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
                component_type,
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_template_id("weather.cloud_layer.inspector")
            .with_data_root("inspector.plugin_components.weather.Component.CloudLayer")
            .with_binding("scene.node.create_cube"),
        )
        .unwrap();
    harness
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("inspector customization extension should register");

    let before = harness.runtime.editor_snapshot().scene_entries.len();
    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&component_drawer_press_envelope(
            component_type,
            "scene.node.create_cube",
        ))
        .expect("declared inspector customization operation should dispatch through host");

    assert!(result.changed);
    assert_eq!(result.mutation_source.as_deref(), Some("component_drawer"));
    assert_eq!(
        result.transaction_id.as_deref(),
        Some("component_drawer:scene.node.create_cube")
    );
    assert_eq!(
        harness.runtime.editor_snapshot().scene_entries.len(),
        before + 1
    );

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&component_drawer_press_envelope(
            component_type,
            "window.layout.reset",
        ))
        .unwrap_err();
    assert_eq!(
        error,
        UiComponentAdapterError::RejectedInput {
            domain: "component_drawer".to_string(),
            path: "window.layout.reset".to_string(),
            reason: "operation is not declared by the enabled inspector customization".to_string(),
        }
    );
}

#[test]
fn inspector_customization_adapter_accepts_safe_action_events_beyond_press() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::extension::InspectorCustomizationDescriptor;

    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_inspector_customization_adapter_safe_events");
    let component_type = "weather.Component.CloudLayer";
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_inspector_customization(
            InspectorCustomizationDescriptor::new(
                component_type,
                "asset://weather/editor/cloud_layer.inspector.zui",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_binding("scene.node.create_cube"),
        )
        .unwrap();
    harness
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("inspector customization extension should register");

    let before = harness.runtime.editor_snapshot().scene_entries.len();
    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&component_drawer_action_envelope(
            component_type,
            "scene.node.create_cube",
            UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::String("apply".to_string()),
            },
        ))
        .expect("commit should dispatch as a safe inspector customization action");

    assert!(result.changed);
    assert_eq!(
        harness.runtime.editor_snapshot().scene_entries.len(),
        before + 1
    );

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&component_drawer_action_envelope(
            component_type,
            "scene.node.create_cube",
            UiComponentEvent::ValueChanged {
                property: "value".to_string(),
                value: UiValue::String("draft".to_string()),
            },
        ))
        .unwrap_err();

    assert_eq!(
        error,
        UiComponentAdapterError::UnsupportedEvent {
            domain: "component_drawer".to_string(),
            path: "scene.node.create_cube".to_string(),
            event_kind: zircon_runtime_interface::ui::component::UiComponentEventKind::ValueChanged,
        }
    );
}
