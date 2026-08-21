use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::UiValue;

use super::support::apply_showcase_binding;
use crate::ui::template_runtime::{EditorUiHostRuntime, UiComponentShowcaseDemoEventInput};

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
