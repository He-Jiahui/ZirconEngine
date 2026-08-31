use super::*;

#[test]
fn editor_component_adapter_registry_advertises_reflection_and_asset_editor_sources() {
    let sources = EditorUiComponentAdapterRegistry::data_sources();

    let inspector = sources
        .iter()
        .find(|source| source.domain == "inspector" && source.source_name == "subject")
        .expect("inspector selected entity source should be advertised");
    assert_eq!(inspector.subject.as_deref(), Some("entity://selected"));
    assert!(inspector.writable);
    assert_eq!(
        inspector
            .fields
            .iter()
            .map(|field| field.path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "name",
            "parent",
            "transform.translation.x",
            "transform.translation.y",
            "transform.translation.z",
        ]
    );

    let reflection_sources = sources
        .iter()
        .filter(|source| source.domain == "reflection")
        .map(|source| source.source_name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        reflection_sources,
        ["asset", "component"].into_iter().collect()
    );
    let component_reflection = sources
        .iter()
        .find(|source| source.domain == "reflection" && source.source_name == "component")
        .expect("component reflection source should be advertised");
    assert!(component_reflection
        .fields
        .iter()
        .any(|field| field.path == "transform.translation" && field.writable));
    assert!(component_reflection.fields.iter().any(|field| {
        field.path == "transform.translation.x"
            && field.writable
            && field.group.as_deref() == Some("Transform")
    }));

    let asset_editor_sources = sources
        .iter()
        .filter(|source| source.domain == "asset_editor")
        .map(|source| (source.source_name.as_str(), source.path_prefix.as_deref()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        asset_editor_sources,
        [
            ("binding", Some("binding")),
            ("layout", Some("layout")),
            ("slot", Some("slot")),
            ("style", Some("style")),
            ("widget", Some("widget")),
        ]
        .into_iter()
        .collect()
    );
    let widget_source = sources
        .iter()
        .find(|source| source.domain == "asset_editor" && source.source_name == "widget")
        .expect("widget asset editor source should be advertised");
    assert!(widget_source
        .fields
        .iter()
        .any(|field| field.path == "widget.text" && field.writable));
    assert!(widget_source
        .fields
        .iter()
        .any(|field| { field.path == "component.root_class_policy" && field.writable }));
}

#[test]
fn editor_event_runtime_exposes_component_data_sources() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_data_sources");
    let sources = harness.runtime.ui_component_data_sources();

    assert!(sources
        .iter()
        .any(|source| source.domain == "reflection" && source.source_name == "component"));
    assert!(sources
        .iter()
        .any(|source| source.domain == "asset_editor" && source.source_name == "binding"));
}

#[test]
fn asset_editor_component_adapter_updates_selected_widget_text() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_asset_component_adapter");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager should be registered");
    let temp_dir = unique_asset_adapter_temp_dir("selected_widget_text");
    fs::create_dir_all(&temp_dir).expect("asset adapter temp dir should be created");
    let asset_path = temp_dir.join("asset-editor-adapter.zui");
    crate::tests::support::write_test_ui_asset(&asset_path, ASSET_EDITOR_ADAPTER_LAYOUT)
        .expect("V2 ui asset adapter fixture should be written");

    let instance_id = manager
        .open_ui_asset_editor(&asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("child widget should be selected");

    let envelope = UiComponentEventEnvelope::new(
        "ui_asset.widget_text",
        "WidgetTextField",
        UiComponentBindingTarget::asset_editor(instance_id.0.clone(), "widget.text"),
        UiComponentEvent::Commit {
            property: "value".to_string(),
            value: UiValue::String("Confirm".to_string()),
        },
    )
    .with_component_id("TextField");

    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .expect("asset editor adapter event should mutate selected widget");
    assert!(result.changed);

    let presentation = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("ui asset editor presentation should refresh");
    assert_eq!(presentation.inspector_text_prop, "Confirm");

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn asset_editor_component_adapter_updates_selected_widget_props_and_state_literals() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_asset_component_props_state_adapter");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager should be registered");
    let temp_dir = unique_asset_adapter_temp_dir("selected_widget_props_state");
    fs::create_dir_all(&temp_dir).expect("asset adapter temp dir should be created");
    let asset_path = temp_dir.join("asset-editor-props-state-adapter.zui");
    crate::tests::support::write_test_ui_asset(&asset_path, ASSET_EDITOR_ADAPTER_LAYOUT)
        .expect("V2 ui asset adapter fixture should be written");

    let instance_id = manager
        .open_ui_asset_editor(&asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("child widget should be selected");

    let prop_envelope = UiComponentEventEnvelope::new(
        "ui_asset.widget_prop",
        "WidgetVariantField",
        UiComponentBindingTarget::asset_editor(instance_id.0.clone(), "widget.prop.variant"),
        UiComponentEvent::Commit {
            property: "value".to_string(),
            value: UiValue::String("primary".to_string()),
        },
    )
    .with_component_id("TextField");
    let state_envelope = UiComponentEventEnvelope::new(
        "ui_asset.widget_state",
        "WidgetExpandedField",
        UiComponentBindingTarget::asset_editor(instance_id.0.clone(), "widget.state.expanded"),
        UiComponentEvent::Commit {
            property: "value".to_string(),
            value: UiValue::Bool(true),
        },
    )
    .with_component_id("Toggle");

    assert!(
        harness
            .runtime
            .dispatch_ui_component_adapter_event(&prop_envelope)
            .expect("asset editor adapter event should mutate selected widget prop")
            .changed
    );
    assert!(
        harness
            .runtime
            .dispatch_ui_component_adapter_event(&state_envelope)
            .expect("asset editor adapter event should mutate selected widget state")
            .changed
    );

    let source = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&source).expect("parse saved ui asset document");
    let selected_node = &document.root.as_ref().unwrap().children[0].node;
    assert_eq!(
        selected_node
            .props
            .get("variant")
            .and_then(toml::Value::as_str),
        Some("primary")
    );
    assert_eq!(
        selected_node
            .params
            .get("expanded")
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn asset_editor_component_adapter_updates_selected_component_root_class_policy() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_asset_component_root_class_adapter");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager should be registered");
    let temp_dir = unique_asset_adapter_temp_dir("component_root_class_policy");
    fs::create_dir_all(&temp_dir).expect("asset adapter temp dir should be created");
    let asset_path = temp_dir.join("asset-editor-component-adapter.zui");
    crate::tests::support::write_test_ui_asset(&asset_path, ASSET_EDITOR_ADAPTER_LAYOUT)
        .expect("V2 ui asset adapter fixture should be written");

    let instance_id = manager
        .open_ui_asset_editor(&asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("child widget should be selected");
    assert!(manager
        .extract_ui_asset_editor_selected_node_to_component(&instance_id)
        .expect("extract selected node to component"));

    let envelope = UiComponentEventEnvelope::new(
        "ui_asset.component_root_class_policy",
        "ComponentRootClassPolicyField",
        UiComponentBindingTarget::asset_editor(
            instance_id.0.clone(),
            "component.root_class_policy",
        ),
        UiComponentEvent::Commit {
            property: "value".to_string(),
            value: UiValue::Enum("closed".to_string()),
        },
    )
    .with_component_id("ComboBox");

    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .expect("asset editor adapter event should mutate component contract");
    assert!(result.changed);

    let source = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&source).expect("parse saved ui asset document");
    assert_eq!(
        document
            .components
            .get("ConfirmButton")
            .map(|component| component.contract.root_class_policy),
        Some(UiRootClassPolicy::Closed)
    );

    let _ = fs::remove_dir_all(temp_dir);
}
