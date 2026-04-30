use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;
use zircon_runtime::ui::component::{UiDragPayload, UiDragPayloadKind, UiValue};

pub(super) fn demo_input_for_showcase_edit(
    action_id: &str,
    value: &str,
) -> UiComponentShowcaseDemoEventInput {
    if action_id.contains("ContextActionMenuOpenAt") {
        if let Some((x, y)) = parse_popup_anchor(value) {
            return UiComponentShowcaseDemoEventInput::OpenPopupAt { x, y };
        }
    }
    if action_id.contains("ArrayFieldRemoveElement") {
        if let Some(index) = value
            .strip_prefix("array-")
            .and_then(|index| index.parse::<usize>().ok())
        {
            return UiComponentShowcaseDemoEventInput::RemoveElement { index };
        }
    }
    if action_id.contains("ArrayFieldMoveElement") {
        if let Some((row_id, to)) = value.split_once('=') {
            if let (Some(from), Some(to)) = (
                row_id
                    .strip_prefix("array-")
                    .and_then(|index| index.parse::<usize>().ok()),
                to.parse::<usize>().ok(),
            ) {
                return UiComponentShowcaseDemoEventInput::MoveElement { from, to };
            }
        }
    }
    if action_id.contains("ArrayFieldSetElement") {
        if let Some((row_id, value)) = value.split_once('=') {
            if let Some(index) = row_id
                .strip_prefix("array-")
                .and_then(|index| index.parse::<usize>().ok())
            {
                return UiComponentShowcaseDemoEventInput::SetElement {
                    index,
                    value: parse_collection_edit_value(value),
                };
            }
        }
    }
    if action_id.contains("MapFieldRemoveEntry") {
        if let Some(key) = value.strip_prefix("map-") {
            return UiComponentShowcaseDemoEventInput::RemoveMapEntry {
                key: key.to_string(),
            };
        }
    }
    if action_id.contains("MapFieldSetEntry") {
        if let Some((row_id, value)) = value.split_once('=') {
            if let Some(key) = row_id.strip_prefix("key:map-") {
                return UiComponentShowcaseDemoEventInput::RenameMapEntry {
                    from_key: key.to_string(),
                    to_key: value.to_string(),
                };
            }
            if let Some(key) = row_id.strip_prefix("map-") {
                return UiComponentShowcaseDemoEventInput::SetMapEntry {
                    key: key.to_string(),
                    value: parse_collection_edit_value(value),
                };
            }
        }
    }
    let value = if action_id.contains("NumberField") || action_id.contains("RangeField") {
        value
            .parse::<f64>()
            .map(UiValue::Float)
            .unwrap_or_else(|_| UiValue::String(value.to_string()))
    } else {
        UiValue::String(value.to_string())
    };
    UiComponentShowcaseDemoEventInput::Value(value)
}

pub(super) fn demo_input_for_showcase_action(
    control_id: &str,
    action_id: &str,
) -> UiComponentShowcaseDemoEventInput {
    match action_id {
        action if action.contains("NumberFieldDragUpdate") => {
            UiComponentShowcaseDemoEventInput::DragDelta(5.0)
        }
        action if action.contains("NumberFieldLargeDragUpdate") => {
            UiComponentShowcaseDemoEventInput::LargeDragDelta(1.0)
        }
        action if action.contains("NumberFieldChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(47.0))
        }
        action if action.contains("RangeFieldChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(72.0))
        }
        action if action.contains("ColorFieldChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Color("#ffcc33".to_string()))
        }
        action if action.contains("Vector2FieldChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Vec2([16.0, 32.0]))
        }
        action if action.contains("Vector3FieldChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Vec3([3.0, 4.0, 5.0]))
        }
        action if action.contains("Vector4FieldChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::Vec4([0.25, 0.5, 0.75, 1.0]))
        }
        action if action.contains("InputField") => UiComponentShowcaseDemoEventInput::Value(
            UiValue::String("Runtime UI event".to_string()),
        ),
        action if action.contains("TextField") => UiComponentShowcaseDemoEventInput::Value(
            UiValue::String("Runtime UI event-driven text".to_string()),
        ),
        action if action.contains("ToggleButtonChanged") || action.contains("CheckboxChanged") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action.contains("RadioChanged") => {
            UiComponentShowcaseDemoEventInput::Toggle(true)
        }
        action if action.contains("SegmentedControlChanged") => select_option("rotate", true),
        action if action.contains("DropdownChanged") => select_option("editor", true),
        action if action.contains("ComboBoxChanged") => select_option("native", true),
        action if action.contains("EnumFieldChanged") => select_option("UnityInspector", true),
        action if action.contains("FlagsFieldChanged") => select_option("Disabled", true),
        action if action.contains("SearchSelectChanged") => {
            select_option("runtime.ui.RangeField", true)
        }
        action if action.contains("SearchSelectQueryChanged") => {
            UiComponentShowcaseDemoEventInput::Value(UiValue::String("vector".to_string()))
        }
        action if action.contains("ContextActionMenuOpenAt") => {
            UiComponentShowcaseDemoEventInput::OpenPopupAt { x: 184.0, y: 88.0 }
        }
        action if action.contains("ContextActionMenuChanged") => select_option("Open Source", true),
        action if action.contains("AssetFieldDropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://materials/runtime_demo.mat",
                ),
            }
        }
        action
            if action.contains("AssetFieldClear")
                || action.contains("AssetFieldLocate")
                || action.contains("AssetFieldOpen") =>
        {
            UiComponentShowcaseDemoEventInput::None
        }
        action if action.contains("InstanceFieldDropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::SceneInstance,
                    "scene://Root/RuntimeDemoLight",
                ),
            }
        }
        action if action.contains("ObjectFieldDropped") => {
            UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Object,
                    "object://Selection/RuntimeDemo",
                ),
            }
        }
        action if action.contains("GroupToggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action.contains("FoldoutToggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(true)
        }
        action if action.contains("InspectorSectionToggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action.contains("TreeRowToggled") => {
            UiComponentShowcaseDemoEventInput::Toggle(false)
        }
        action if action.contains("ArrayFieldAddElement") => {
            UiComponentShowcaseDemoEventInput::AddElement {
                value: UiValue::String("MapField".to_string()),
            }
        }
        action if action.contains("ArrayFieldSetElement") => {
            UiComponentShowcaseDemoEventInput::SetElement {
                index: 1,
                value: UiValue::String("Vector3Field".to_string()),
            }
        }
        action if action.contains("ArrayFieldRemoveElement") => {
            UiComponentShowcaseDemoEventInput::RemoveElement { index: 0 }
        }
        action if action.contains("ArrayFieldMoveElement") => {
            UiComponentShowcaseDemoEventInput::MoveElement { from: 0, to: 1 }
        }
        action if action.contains("MapFieldAddEntry") => {
            UiComponentShowcaseDemoEventInput::AddMapEntry {
                key: "layer".to_string(),
                value: UiValue::String("Editor".to_string()),
            }
        }
        action if action.contains("MapFieldSetEntry") => {
            UiComponentShowcaseDemoEventInput::SetMapEntry {
                key: "speed".to_string(),
                value: UiValue::Float(2.5),
            }
        }
        action if action.contains("MapFieldRemoveEntry") => {
            UiComponentShowcaseDemoEventInput::RemoveMapEntry {
                key: "speed".to_string(),
            }
        }
        action if action.contains("ListRowClicked") => UiComponentShowcaseDemoEventInput::None,
        action if action.contains("Show") && control_id.starts_with("ComponentShowcase") => {
            UiComponentShowcaseDemoEventInput::None
        }
        _ => UiComponentShowcaseDemoEventInput::None,
    }
}

fn parse_collection_edit_value(value: &str) -> UiValue {
    if let Ok(value) = value.parse::<bool>() {
        return UiValue::Bool(value);
    }
    value
        .parse::<f64>()
        .map(UiValue::Float)
        .unwrap_or_else(|_| UiValue::String(value.to_string()))
}

fn parse_popup_anchor(value: &str) -> Option<(f64, f64)> {
    let (x, y) = value.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

pub(super) fn select_option(option_id: &str, selected: bool) -> UiComponentShowcaseDemoEventInput {
    UiComponentShowcaseDemoEventInput::SelectOption {
        option_id: option_id.to_string(),
        selected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn showcase_edit_input_maps_collection_row_payloads_to_typed_events() {
        assert_eq!(
            demo_input_for_showcase_edit(
                "UiComponentShowcase/ArrayFieldSetElement",
                "array-0=Vector3Field",
            ),
            UiComponentShowcaseDemoEventInput::SetElement {
                index: 0,
                value: UiValue::String("Vector3Field".to_string()),
            }
        );
        assert_eq!(
            demo_input_for_showcase_edit("UiComponentShowcase/MapFieldSetEntry", "map-speed=2.5"),
            UiComponentShowcaseDemoEventInput::SetMapEntry {
                key: "speed".to_string(),
                value: UiValue::Float(2.5),
            }
        );
        assert_eq!(
            demo_input_for_showcase_edit(
                "UiComponentShowcase/MapFieldSetEntry",
                "map-visible=false"
            ),
            UiComponentShowcaseDemoEventInput::SetMapEntry {
                key: "visible".to_string(),
                value: UiValue::Bool(false),
            }
        );
        assert_eq!(
            demo_input_for_showcase_edit(
                "UiComponentShowcase/MapFieldSetEntry",
                "key:map-speed=velocity",
            ),
            UiComponentShowcaseDemoEventInput::RenameMapEntry {
                from_key: "speed".to_string(),
                to_key: "velocity".to_string(),
            }
        );
        assert_eq!(
            demo_input_for_showcase_edit("UiComponentShowcase/ArrayFieldMoveElement", "array-2=1",),
            UiComponentShowcaseDemoEventInput::MoveElement { from: 2, to: 1 }
        );
        assert_eq!(
            demo_input_for_showcase_edit("UiComponentShowcase/ArrayFieldRemoveElement", "array-1",),
            UiComponentShowcaseDemoEventInput::RemoveElement { index: 1 }
        );
        assert_eq!(
            demo_input_for_showcase_edit("UiComponentShowcase/MapFieldRemoveEntry", "map-visible",),
            UiComponentShowcaseDemoEventInput::RemoveMapEntry {
                key: "visible".to_string(),
            }
        );
        assert_eq!(
            demo_input_for_showcase_edit("UiComponentShowcase/ContextActionMenuOpenAt", "212,96",),
            UiComponentShowcaseDemoEventInput::OpenPopupAt { x: 212.0, y: 96.0 }
        );
        assert_eq!(
            demo_input_for_showcase_edit("UiComponentShowcase/InputFieldCommitted", "committed"),
            UiComponentShowcaseDemoEventInput::Value(UiValue::String("committed".to_string()))
        );
        assert_eq!(
            demo_input_for_showcase_edit("UiComponentShowcase/NumberFieldCommitted", "51"),
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(51.0))
        );
    }
}
