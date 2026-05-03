use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope,
    UiValue, UiValueKind,
};
use zircon_runtime_interface::ui::template::UiRootClassPolicy;

use crate::ui::template_runtime::component_adapter::registry::EditorUiComponentAdapterRegistry;

fn inspector_value_envelope(field_path: &str, value: UiValue) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "inspector.surface_controls",
        field_path.replace('.', "_"),
        UiComponentBindingTarget::inspector("entity://selected", field_path),
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value,
        },
    )
    .with_component_id("InspectorField")
}

fn inspector_commit_envelope(field_path: &str, value: UiValue) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "inspector.surface_controls",
        field_path.replace('.', "_"),
        UiComponentBindingTarget::inspector("entity://selected", field_path),
        UiComponentEvent::Commit {
            property: "value".to_string(),
            value,
        },
    )
    .with_component_id("InspectorField")
}

fn reflection_commit_envelope(field_path: &str, value: UiValue) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "reflection.surface_controls",
        field_path.replace('.', "_"),
        UiComponentBindingTarget::reflection("component://selected", field_path),
        UiComponentEvent::Commit {
            property: "value".to_string(),
            value,
        },
    )
    .with_component_id("ReflectionField")
}

fn component_drawer_press_envelope(
    component_type: &str,
    operation_path: &str,
) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        "asset://weather/editor/cloud_layer.inspector.ui.toml",
        "RefreshButton",
        UiComponentBindingTarget::new("component_drawer", operation_path)
            .with_subject(component_type),
        UiComponentEvent::Press { pressed: true },
    )
    .with_component_id("weather.cloud_layer.inspector")
}

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
fn component_drawer_adapter_invokes_only_enabled_declared_operation_bindings() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};

    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_drawer_adapter_operation");
    let component_type = "weather.Component.CloudLayer";
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                component_type,
                "asset://weather/editor/cloud_layer.inspector.ui.toml",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_template_id("weather.cloud_layer.inspector")
            .with_data_root("inspector.plugin_components.weather.Component.CloudLayer")
            .with_binding("Scene.Node.CreateCube"),
        )
        .unwrap();
    harness
        .runtime
        .register_editor_extension(extension)
        .expect("component drawer extension should register");

    let before = harness.runtime.editor_snapshot().scene_entries.len();
    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&component_drawer_press_envelope(
            component_type,
            "Scene.Node.CreateCube",
        ))
        .expect("declared component drawer operation should dispatch through host");

    assert!(result.changed);
    assert_eq!(result.mutation_source.as_deref(), Some("component_drawer"));
    assert_eq!(
        result.transaction_id.as_deref(),
        Some("component_drawer:Scene.Node.CreateCube")
    );
    assert_eq!(harness.runtime.editor_snapshot().scene_entries.len(), before + 1);

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&component_drawer_press_envelope(
            component_type,
            "Window.Layout.Reset",
        ))
        .unwrap_err();
    assert_eq!(
        error,
        UiComponentAdapterError::RejectedInput {
            domain: "component_drawer".to_string(),
            path: "Window.Layout.Reset".to_string(),
            reason: "operation is not declared by the enabled component drawer".to_string(),
        }
    );
}

#[test]
fn inspector_component_adapter_rejects_missing_selection_without_mutation() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_missing_subject");
    let before = harness.runtime.editor_snapshot().inspector.unwrap().name;
    let envelope = UiComponentEventEnvelope::new(
        "inspector.surface_controls",
        "NameField",
        UiComponentBindingTarget::new("inspector", "name"),
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("Should Not Apply".to_string()),
        },
    )
    .with_component_id("InspectorField");

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert_eq!(
        error,
        UiComponentAdapterError::MissingSource {
            domain: "inspector".to_string(),
            path: "name".to_string(),
            source_name: "subject".to_string(),
        }
    );
    assert_eq!(
        harness.runtime.editor_snapshot().inspector.unwrap().name,
        before
    );
}

#[test]
fn inspector_component_adapter_rejects_unsupported_field() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_unsupported_field");
    let before = harness.runtime.editor_snapshot().inspector.unwrap();
    let envelope = inspector_value_envelope("transform.rotation.x", UiValue::Array(Vec::new()));

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert_eq!(
        error,
        UiComponentAdapterError::UnsupportedTargetPath {
            domain: "inspector".to_string(),
            path: "transform.rotation.x".to_string(),
        }
    );
    let after = harness.runtime.editor_snapshot().inspector.unwrap();
    assert_eq!(after.name, before.name);
    assert_eq!(after.translation, before.translation);
}

#[test]
fn inspector_component_adapter_rejects_invalid_value_kind_for_supported_field() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_invalid_value_kind");
    let before = harness.runtime.editor_snapshot().inspector.unwrap();
    let envelope = inspector_value_envelope("name", UiValue::Bool(true));

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert_eq!(
        error,
        UiComponentAdapterError::InvalidValueKind {
            domain: "inspector".to_string(),
            path: "name".to_string(),
            expected: UiValueKind::String,
            actual: UiValueKind::Bool,
        }
    );
    assert_eq!(
        harness.runtime.editor_snapshot().inspector.unwrap().name,
        before.name
    );
}

#[test]
fn inspector_component_adapter_rejects_non_value_property_without_mutation() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_invalid_property");
    let before = harness.runtime.editor_snapshot().inspector.unwrap();
    let envelope = UiComponentEventEnvelope::new(
        "inspector.surface_controls",
        "NameField",
        UiComponentBindingTarget::inspector("entity://selected", "name"),
        UiComponentEvent::ValueChanged {
            property: "label".to_string(),
            value: UiValue::String("Should Not Apply".to_string()),
        },
    )
    .with_component_id("InspectorField");

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert!(matches!(
        error,
        UiComponentAdapterError::RejectedInput { .. }
    ));
    assert_eq!(
        harness.runtime.editor_snapshot().inspector.unwrap().name,
        before.name
    );
}

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
    let asset_path = temp_dir.join("asset-editor-adapter.ui.toml");
    write_text_file(&asset_path, ASSET_EDITOR_ADAPTER_LAYOUT);

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
fn asset_editor_component_adapter_updates_selected_component_root_class_policy() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_asset_component_root_class_adapter");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager should be registered");
    let temp_dir = unique_asset_adapter_temp_dir("component_root_class_policy");
    fs::create_dir_all(&temp_dir).expect("asset adapter temp dir should be created");
    let asset_path = temp_dir.join("asset-editor-component-adapter.ui.toml");
    write_text_file(&asset_path, ASSET_EDITOR_ADAPTER_LAYOUT);

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

const ASSET_EDITOR_ADAPTER_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "editor.tests.asset.component_adapter"
version = 1
display_name = "Component Adapter UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "confirm" }]

[nodes.confirm]
kind = "native"
type = "Button"
control_id = "ConfirmButton"
props = { text = "Save" }
"#;

fn unique_asset_adapter_temp_dir(suffix: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis();
    std::env::temp_dir().join(format!(
        "zircon-ui-asset-component-adapter-{suffix}-{millis}"
    ))
}

fn write_text_file(path: &Path, contents: &str) {
    fs::write(path, contents).expect("test file should be written");
}
