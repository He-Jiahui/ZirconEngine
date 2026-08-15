use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use super::support::*;
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiHostValue};

#[test]
fn host_projection_carries_runtime_component_properties_and_routes() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime.load_builtin_host_templates().unwrap();

    let document_id = "res://ui/editor/host/inspector_surface_controls.zui";
    let projection = ui_runtime.project_document(document_id).unwrap();
    let surface = ui_runtime.build_shared_surface(document_id).unwrap();
    let host_projection = ui_runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let name_field = host_projection
        .node_by_control_id("NameField")
        .expect("inspector surface should project NameField");
    assert_eq!(name_field.component, "InputField");
    assert_eq!(name_field.text.as_deref(), None);
    assert_eq!(
        name_field.properties.get("placeholder"),
        Some(&RetainedUiHostValue::String("Name".to_string()))
    );
    assert_host_numeric_property(name_field, "layout_min_height", 28.0);
    assert_host_bool_property(name_field, "input_focusable", true);
    assert!(name_field.routes.iter().any(|route| {
        route.binding_id == "InspectorView/NameField" && route.event_kind == UiEventKind::Change
    }));

    let position_x = host_projection
        .node_by_control_id("PositionXField")
        .expect("inspector surface should project PositionXField");
    assert_eq!(position_x.component, "NumberField");
    assert_eq!(position_x.value_text.as_deref(), Some("0"));
    assert_host_numeric_property(position_x, "layout_min_height", 28.0);
    assert_host_bool_property(position_x, "input_focusable", true);
    assert!(position_x.routes.iter().any(|route| {
        route.binding_id == "InspectorView/PositionXField"
            && route.event_kind == UiEventKind::Change
    }));

    let apply_button = host_projection
        .node_by_control_id("ApplyBatchButton")
        .expect("inspector surface should project ApplyBatchButton");
    assert_eq!(apply_button.component, "Button");
    assert_eq!(apply_button.text.as_deref(), Some("Apply"));
    assert_host_numeric_property(apply_button, "layout_min_height", 28.0);
    assert_host_bool_property(apply_button, "input_clickable", true);
    assert!(apply_button.routes.iter().any(|route| {
        route.binding_id == "InspectorView/ApplyBatchButton"
            && route.event_kind == UiEventKind::Click
    }));
}
