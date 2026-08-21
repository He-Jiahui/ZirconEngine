use zircon_runtime_interface::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata, UiValue,
};

use super::support::{apply_showcase_binding, showcase_binding};
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostValue, UiComponentShowcaseDemoEventInput,
};

#[test]
fn showcase_demo_state_applies_projected_bindings_to_retained_values_and_log() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowDataCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    assert_eq!(
        runtime.showcase_demo_state().selected_category(),
        "Collections"
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowAllCategory",
        UiComponentShowcaseDemoEventInput::None,
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ButtonCommit",
        UiComponentShowcaseDemoEventInput::None,
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ButtonDemo", "value")
            .as_deref(),
        Some("")
    );

    let result = apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/InputFieldChanged",
        UiComponentShowcaseDemoEventInput::Value(UiValue::String("hello runtime".to_string())),
    );
    assert!(result.changed);
    assert!(result.refresh_projection);
    assert!(result
        .patches
        .iter()
        .any(|patch| patch.control_id == "InputFieldDemo"));
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("InputFieldDemo", "value")
            .as_deref(),
        Some("hello runtime")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/InputFieldCommitted",
        UiComponentShowcaseDemoEventInput::Value(UiValue::String("committed runtime".to_string())),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("InputFieldDemo", "value")
            .as_deref(),
        Some("committed runtime")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/CheckboxChanged",
        UiComponentShowcaseDemoEventInput::Toggle(false),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("CheckboxDemo", "value")
            .as_deref(),
        Some("false")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/NumberFieldDragUpdate",
        UiComponentShowcaseDemoEventInput::DragDelta(5.0),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("NumberFieldDemo", "value")
            .as_deref(),
        Some("47")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/NumberFieldCommitted",
        UiComponentShowcaseDemoEventInput::Value(UiValue::Float(51.0)),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("NumberFieldDemo", "value")
            .as_deref(),
        Some("51")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ColorFieldChanged",
        UiComponentShowcaseDemoEventInput::Value(UiValue::Color("#ffcc33".to_string())),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ColorFieldDemo", "value")
            .as_deref(),
        Some("#ffcc33")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/Vector3FieldChanged",
        UiComponentShowcaseDemoEventInput::Value(UiValue::Vec3([3.0, 4.0, 5.0])),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("Vector3FieldDemo", "value")
            .as_deref(),
        Some("3, 4, 5")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/DropdownChanged",
        UiComponentShowcaseDemoEventInput::SelectOption {
            option_id: "editor".to_string(),
            selected: true,
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("DropdownDemo", "value")
            .as_deref(),
        Some("2 items")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ComboBoxOpenPopup",
        UiComponentShowcaseDemoEventInput::None,
    );
    let projection = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert!(
        host_projection
            .node_by_control_id("ComboBoxDemo")
            .is_some_and(|node| node.popup_open),
        "OpenPopup should be retained and projected for ComboBoxDemo"
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ComboBoxClosePopup",
        UiComponentShowcaseDemoEventInput::None,
    );
    let projection = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert!(
        host_projection
            .node_by_control_id("ComboBoxDemo")
            .is_some_and(|node| !node.popup_open),
        "ClosePopup should be retained and projected for ComboBoxDemo"
    );
    let header = host_projection
        .node_by_control_id("UiComponentShowcaseHeader")
        .expect("showcase header should project");
    assert!(header.text.as_deref().is_some_and(|text| {
        text.contains("material_dark / fyrox_panel / jetbrains_shell / unreal_window_model")
    }));
    assert_eq!(
        header.properties.get("text_tone"),
        Some(&RetainedUiHostValue::String("default".to_string()))
    );

    let source = UiDragSourceMetadata::asset(
        "browser",
        "AssetBrowserContentPanel",
        "asset-uuid-1",
        "res://materials/demo.mat",
        "Demo Material",
        "Material",
        "mat",
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldDropped",
        UiComponentShowcaseDemoEventInput::DropReference {
            payload: UiDragPayload::new(UiDragPayloadKind::Asset, "res://materials/demo.mat")
                .with_source(source),
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("res://materials/demo.mat")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ArrayFieldAddElement",
        UiComponentShowcaseDemoEventInput::AddElement {
            value: UiValue::String("MapField".to_string()),
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ArrayFieldDemo", "items")
            .as_deref(),
        Some("4 items")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ArrayFieldMoveElement",
        UiComponentShowcaseDemoEventInput::MoveElement { from: 3, to: 0 },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ArrayFieldDemo", "items")
            .as_deref(),
        Some("4 items")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ArrayFieldRemoveElement",
        UiComponentShowcaseDemoEventInput::RemoveElement { index: 1 },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ArrayFieldDemo", "items")
            .as_deref(),
        Some("3 items")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/MapFieldAddEntry",
        UiComponentShowcaseDemoEventInput::AddMapEntry {
            key: "layer".to_string(),
            value: UiValue::String("Editor".to_string()),
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("MapFieldDemo", "entries")
            .as_deref(),
        Some("3 entries")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/MapFieldRemoveEntry",
        UiComponentShowcaseDemoEventInput::RemoveMapEntry {
            key: "speed".to_string(),
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("MapFieldDemo", "entries")
            .as_deref(),
        Some("2 entries")
    );

    assert!(
        runtime
            .showcase_demo_state()
            .event_log()
            .iter()
            .any(|entry| entry.action == "DragDelta.NumberField"
                && entry.control_id == "NumberFieldDemo"),
        "state reducer should append a typed event-log entry for projected bindings"
    );
    assert!(
        runtime
            .showcase_demo_state()
            .event_log()
            .iter()
            .any(|entry| entry.action == "Commit.InputField"
                && entry.control_id == "InputFieldDemo"
                && entry.value_text.as_deref() == Some("committed runtime")),
        "committed text edits should be logged as typed Runtime UI commit events"
    );
    assert!(
        runtime
            .showcase_demo_state()
            .event_log()
            .iter()
            .any(|entry| entry.action == "Commit.NumberField"
                && entry.control_id == "NumberFieldDemo"
                && entry.value_text.as_deref() == Some("51")),
        "committed numeric edits should be logged as typed Runtime UI commit events"
    );

    let projection = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();

    assert_eq!(
        host_projection
            .node_by_control_id("NumberFieldDemo")
            .and_then(|node| node.value_text.as_deref()),
        Some("51")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("DropdownDemo")
            .and_then(|node| node.value_text.as_deref()),
        Some("2 items")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ColorFieldDemo")
            .and_then(|node| node.value_text.as_deref()),
        Some("#ffcc33")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ColorFieldDemo")
            .and_then(|node| node.properties.get("value")),
        Some(&RetainedUiHostValue::String("#ffcc33".to_string()))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("Vector3FieldDemo")
            .and_then(|node| node.value_text.as_deref()),
        Some("3, 4, 5")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("Vector3FieldDemo")
            .and_then(|node| node.properties.get("value")),
        Some(&RetainedUiHostValue::Array(vec![
            RetainedUiHostValue::Float(3.0),
            RetainedUiHostValue::Float(4.0),
            RetainedUiHostValue::Float(5.0),
        ]))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .and_then(|node| node.value_text.as_deref()),
        Some("res://materials/demo.mat")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .and_then(|node| node.properties.get("drop_source_summary")),
        Some(&RetainedUiHostValue::String(
            "Material: Demo Material".to_string()
        ))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .and_then(|node| node.drop_source_summary.as_deref()),
        Some("Material: Demo Material")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .and_then(|node| node.properties.get("value")),
        Some(&RetainedUiHostValue::String(
            "res://materials/demo.mat".to_string()
        ))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ComponentShowcaseLastControl")
            .and_then(|node| node.value_text.as_deref()),
        Some("MapFieldDemo")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ComponentShowcaseLastAction")
            .and_then(|node| node.value_text.as_deref()),
        Some("RemoveMapEntry.MapField")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ComponentShowcaseCurrentValue")
            .and_then(|node| node.value_text.as_deref()),
        Some("2 entries")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ComponentShowcaseValidation")
            .and_then(|node| node.value_text.as_deref()),
        Some("normal")
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ComponentShowcaseDragPayload")
            .and_then(|node| node.value_text.as_deref()),
        Some("No retained drop payload")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldClear",
        UiComponentShowcaseDemoEventInput::None,
    );
    let projection = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert_eq!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .and_then(|node| node.properties.get("drop_source_summary")),
        None
    );
    assert_eq!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .and_then(|node| node.drop_source_summary.as_deref()),
        None
    );
    assert_eq!(
        host_projection
            .node_by_control_id("MapFieldDemo")
            .and_then(|node| node.value_text.as_deref()),
        Some("2 entries")
    );
    assert!(
        host_projection
            .node_by_control_id("ComponentShowcaseEventLog")
            .and_then(|node| node.text.as_deref())
            .is_some_and(
                |text| text.contains("MapFieldDemo -> RemoveMapEntry.MapField = 2 entries")
            ),
        "event log label should be rebuilt from retained showcase state"
    );

    let binding = showcase_binding(&runtime, "UiComponentShowcase/ColorFieldChanged");
    let error = runtime
        .apply_showcase_demo_binding(
            &binding,
            UiComponentShowcaseDemoEventInput::Value(UiValue::String("#not-a-color".to_string())),
        )
        .unwrap_err();
    assert!(error.to_string().contains("invalid value kind"));

    let projection = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert_eq!(
        host_projection
            .node_by_control_id("ComponentShowcaseLastControl")
            .and_then(|node| node.value_text.as_deref()),
        Some("ColorFieldDemo")
    );
    assert!(
        host_projection
            .node_by_control_id("ComponentShowcaseValidation")
            .and_then(|node| node.value_text.as_deref())
            .is_some_and(|value| value.contains("invalid value kind")),
        "failed retained events should select the failed control in the state panel"
    );
}
