use std::collections::BTreeMap;

use zircon_runtime::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata, UiValue,
};

use crate::ui::template_runtime::{
    EditorUiHostRuntime, SlintUiHostValue, UiComponentShowcaseDemoEventInput,
};

fn showcase_binding(
    runtime: &EditorUiHostRuntime,
    binding_id: &str,
) -> crate::ui::binding::EditorUiBinding {
    runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap()
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == binding_id)
        .unwrap_or_else(|| panic!("missing showcase binding `{binding_id}`"))
        .binding
}

fn apply_showcase_binding(
    runtime: &mut EditorUiHostRuntime,
    binding_id: &str,
    input: UiComponentShowcaseDemoEventInput,
) {
    let binding = showcase_binding(runtime, binding_id);
    runtime
        .apply_showcase_demo_binding(&binding, input)
        .unwrap();
}

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

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/InputFieldChanged",
        UiComponentShowcaseDemoEventInput::Value(UiValue::String("hello runtime".to_string())),
    );
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
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
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
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert!(
        host_projection
            .node_by_control_id("ComboBoxDemo")
            .is_some_and(|node| !node.popup_open),
        "ClosePopup should be retained and projected for ComboBoxDemo"
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
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
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
        Some(&SlintUiHostValue::String("#ffcc33".to_string()))
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
        Some(&SlintUiHostValue::Array(vec![
            SlintUiHostValue::Float(3.0),
            SlintUiHostValue::Float(4.0),
            SlintUiHostValue::Float(5.0),
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
        Some(&SlintUiHostValue::String(
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
        Some(&SlintUiHostValue::String(
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
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
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
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
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

#[test]
fn showcase_demo_state_exercises_full_component_action_bindings() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/NumberFieldLargeDragUpdate",
        UiComponentShowcaseDemoEventInput::LargeDragDelta(1.0),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("NumberFieldDemo", "value")
            .as_deref(),
        Some("52")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldLocate",
        UiComponentShowcaseDemoEventInput::None,
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldOpen",
        UiComponentShowcaseDemoEventInput::None,
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldClear",
        UiComponentShowcaseDemoEventInput::None,
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ArrayFieldSetElement",
        UiComponentShowcaseDemoEventInput::SetElement {
            index: 1,
            value: UiValue::String("Vector3Field".to_string()),
        },
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
        "UiComponentShowcase/ArrayFieldChanged",
        UiComponentShowcaseDemoEventInput::Value(UiValue::Array(vec![UiValue::String(
            "OnlyChild".to_string(),
        )])),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ArrayFieldDemo", "items")
            .as_deref(),
        Some("1 items")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/MapFieldSetEntry",
        UiComponentShowcaseDemoEventInput::SetMapEntry {
            key: "speed".to_string(),
            value: UiValue::Float(2.5),
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("MapFieldDemo", "entries")
            .as_deref(),
        Some("2 entries")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/MapFieldSetEntry",
        UiComponentShowcaseDemoEventInput::RenameMapEntry {
            from_key: "speed".to_string(),
            to_key: "velocity".to_string(),
        },
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("MapFieldDemo", "entries")
            .as_deref(),
        Some("2 entries")
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/InspectorSectionToggled",
        UiComponentShowcaseDemoEventInput::Toggle(false),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("InspectorSectionDemo", "expanded")
            .as_deref(),
        Some("false")
    );
    let projection = runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert!(
        host_projection
            .node_by_control_id("MapFieldDemo")
            .expect("MapFieldDemo")
            .collection_items
            .iter()
            .any(|item| item.starts_with("velocity: String -> UiValue = 2.5")),
        "MapField key edits should rename the retained key used by projected child rows"
    );
    assert!(
        host_projection
            .node_by_control_id("InspectorSectionDemo")
            .is_some_and(|node| !node.expanded),
        "InspectorSection ToggleExpanded should override the authored expanded state"
    );

    let log = runtime.showcase_demo_state().event_log();
    assert!(log
        .iter()
        .any(|entry| entry.action == "LargeDragDelta.NumberField"));
    assert!(log
        .iter()
        .any(|entry| entry.action == "ClearReference.AssetField"));
    assert!(log
        .iter()
        .any(|entry| entry.action == "SetMapEntry.MapField"));

    let mut replacement_entries = BTreeMap::new();
    replacement_entries.insert("replacement".to_string(), UiValue::Bool(true));
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/MapFieldChanged",
        UiComponentShowcaseDemoEventInput::Value(UiValue::Map(replacement_entries)),
    );
    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("MapFieldDemo", "entries")
            .as_deref(),
        Some("1 entries")
    );
}

#[test]
fn showcase_demo_state_projects_collection_children_and_control_flags() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowClicked",
        UiComponentShowcaseDemoEventInput::None,
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowHovered",
        UiComponentShowcaseDemoEventInput::Hover(true),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowPressed",
        UiComponentShowcaseDemoEventInput::Press(true),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/NumberFieldDragBegin",
        UiComponentShowcaseDemoEventInput::None,
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldDropHovered",
        UiComponentShowcaseDemoEventInput::DropHover(true),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldActiveDragTarget",
        UiComponentShowcaseDemoEventInput::ActiveDragTarget(true),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ContextActionMenuOpenPopup",
        UiComponentShowcaseDemoEventInput::None,
    );

    let projection = runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let list_row = host_projection
        .node_by_control_id("ListRowDemo")
        .expect("ListRowDemo should be projected");
    assert!(list_row.focused, "ListRow should retain focus state");
    assert_eq!(
        list_row.selection_state.as_deref(),
        Some("focused"),
        "focused row state should be represented as a selection-state token"
    );
    assert!(list_row.hovered, "ListRow should retain hover state");
    assert!(list_row.pressed, "ListRow should retain press state");

    assert!(
        host_projection
            .node_by_control_id("NumberFieldDemo")
            .is_some_and(|node| node.dragging),
        "NumberField BeginDrag should be retained and projected"
    );
    assert!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .is_some_and(|node| node.drop_hovered),
        "AssetField DropHover should be retained and projected"
    );
    assert!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .is_some_and(|node| node.active_drag_target),
        "AssetField ActiveDragTarget should be retained and projected"
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowHovered",
        UiComponentShowcaseDemoEventInput::Hover(false),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowPressed",
        UiComponentShowcaseDemoEventInput::Press(false),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldDropHovered",
        UiComponentShowcaseDemoEventInput::DropHover(false),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldActiveDragTarget",
        UiComponentShowcaseDemoEventInput::ActiveDragTarget(false),
    );
    let projection = runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let list_row = host_projection
        .node_by_control_id("ListRowDemo")
        .expect("ListRowDemo should be projected after transient flags clear");
    assert!(
        !list_row.hovered,
        "ListRow Hover(false) should override the authored showcase hover prop"
    );
    assert!(
        !list_row.pressed,
        "ListRow Press(false) should clear retained press state"
    );
    let asset_field = host_projection
        .node_by_control_id("AssetFieldDemo")
        .expect("AssetFieldDemo should be projected after transient flags clear");
    assert!(
        !asset_field.drop_hovered,
        "AssetField DropHover(false) should override the authored showcase drop-hover prop"
    );
    assert!(
        !asset_field.active_drag_target,
        "AssetField ActiveDragTarget(false) should override the authored showcase active target prop"
    );

    assert_eq!(
        host_projection
            .node_by_control_id("ArrayFieldDemo")
            .expect("ArrayFieldDemo")
            .collection_items,
        vec![
            "#0 UiComponentRef = Label".to_string(),
            "#1 UiComponentRef = NumberField".to_string(),
            "#2 UiComponentRef = AssetField".to_string(),
        ],
        "ArrayField should project generated child rows from its element schema"
    );
    assert_eq!(
        host_projection
            .node_by_control_id("MapFieldDemo")
            .expect("MapFieldDemo")
            .collection_items,
        vec![
            "speed: String -> UiValue = 1".to_string(),
            "visible: String -> UiValue = true".to_string(),
        ],
        "MapField should project generated key/value child rows from its typed schema"
    );

    let menu = host_projection
        .node_by_control_id("ContextActionMenuDemo")
        .expect("ContextActionMenuDemo");
    assert!(menu.popup_open);
    assert_eq!(
        menu.menu_items,
        vec![
            "Inspect|checked,focused|Ctrl+I".to_string(),
            "---".to_string(),
            "Duplicate|hovered,pressed|Ctrl+D".to_string(),
            "Delete|disabled|Del".to_string(),
        ],
        "ContextActionMenu should project menu-row metadata beyond a flat option label"
    );
}

#[test]
fn showcase_context_action_menu_selects_clean_action_labels_from_menu_metadata() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ContextActionMenuChanged",
        UiComponentShowcaseDemoEventInput::SelectOption {
            option_id: "Duplicate||Ctrl+D".to_string(),
            selected: true,
        },
    );

    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ContextActionMenuDemo", "value")
            .as_deref(),
        Some("Duplicate"),
        "ContextActionMenu should store the selected action label, not the encoded menu-row metadata"
    );
}

#[test]
fn showcase_context_action_menu_opens_at_retained_pointer_anchor() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ContextActionMenuOpenAt",
        UiComponentShowcaseDemoEventInput::OpenPopupAt { x: 212.0, y: 96.0 },
    );

    let projection = runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let menu = host_projection
        .node_by_control_id("ContextActionMenuDemo")
        .expect("ContextActionMenuDemo");
    assert!(menu.popup_open);
    assert!(menu.has_popup_anchor);
    assert_eq!(menu.popup_anchor_x, 212.0);
    assert_eq!(menu.popup_anchor_y, 96.0);
    assert_eq!(
        menu.properties.get("popup_anchor_x"),
        Some(&SlintUiHostValue::Float(212.0))
    );
    assert_eq!(
        menu.properties.get("popup_anchor_y"),
        Some(&SlintUiHostValue::Float(96.0))
    );
}
