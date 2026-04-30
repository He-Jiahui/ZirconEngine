use std::collections::BTreeMap;

use zircon_runtime::ui::component::{UiComponentState, UiValue};

pub(super) fn component_id_for_control(control_id: &str) -> Option<&'static str> {
    match control_id {
        "ButtonDemo" => Some("Button"),
        "IconButtonDemo" => Some("IconButton"),
        "ToggleButtonDemo" => Some("ToggleButton"),
        "CheckboxDemo" => Some("Checkbox"),
        "RadioDemo" => Some("Radio"),
        "SegmentedControlDemo" => Some("SegmentedControl"),
        "InputFieldDemo" => Some("InputField"),
        "TextFieldDemo" => Some("TextField"),
        "NumberFieldDemo" => Some("NumberField"),
        "RangeFieldDemo" => Some("RangeField"),
        "ColorFieldDemo" => Some("ColorField"),
        "Vector2FieldDemo" => Some("Vector2Field"),
        "Vector3FieldDemo" => Some("Vector3Field"),
        "Vector4FieldDemo" => Some("Vector4Field"),
        "DropdownDemo" => Some("Dropdown"),
        "ComboBoxDemo" => Some("ComboBox"),
        "EnumFieldDemo" => Some("EnumField"),
        "FlagsFieldDemo" => Some("FlagsField"),
        "SearchSelectDemo" => Some("SearchSelect"),
        "AssetFieldDemo" => Some("AssetField"),
        "InstanceFieldDemo" => Some("InstanceField"),
        "ObjectFieldDemo" => Some("ObjectField"),
        "GroupDemo" => Some("Group"),
        "FoldoutDemo" => Some("Foldout"),
        "PropertyRowDemo" => Some("PropertyRow"),
        "InspectorSectionDemo" => Some("InspectorSection"),
        "ArrayFieldDemo" => Some("ArrayField"),
        "MapFieldDemo" => Some("MapField"),
        "ListRowDemo" => Some("ListRow"),
        "TreeRowDemo" => Some("TreeRow"),
        "ContextActionMenuDemo" => Some("ContextActionMenu"),
        _ => None,
    }
}

pub(super) fn default_state_for_control(control_id: &str) -> UiComponentState {
    match control_id {
        "NumberFieldDemo" => UiComponentState::new().with_value("value", UiValue::Float(42.0)),
        "RangeFieldDemo" => UiComponentState::new().with_value("value", UiValue::Float(68.0)),
        "ColorFieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Color("#4d89ff".to_string()))
        }
        "Vector2FieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Vec2([12.0, 24.0]))
        }
        "Vector3FieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Vec3([0.0, 1.0, 0.0]))
        }
        "Vector4FieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Vec4([0.0, 1.0, 0.0, 1.0]))
        }
        "DropdownDemo" => UiComponentState::new()
            .with_value("value", UiValue::Enum("runtime".to_string()))
            .with_value("multiple", UiValue::Bool(true))
            .with_value(
                "disabled_options",
                UiValue::Array(vec![UiValue::String("debug".to_string())]),
            ),
        "ComboBoxDemo" => {
            UiComponentState::new().with_value("value", UiValue::Enum("material".to_string()))
        }
        "EnumFieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Enum("RiderDocking".to_string()))
        }
        "FlagsFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::Flags(vec!["Selectable".to_string(), "Draggable".to_string()]),
        ),
        "SearchSelectDemo" => UiComponentState::new()
            .with_value("value", UiValue::Enum("runtime.ui.NumberField".to_string()))
            .with_value("query", UiValue::String("number".to_string())),
        "AssetFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::AssetRef("res://textures/grid.albedo.png".to_string()),
        ),
        "InstanceFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::InstanceRef("scene://Root/CameraRig".to_string()),
        ),
        "ObjectFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::InstanceRef("object://Selection/MainCamera".to_string()),
        ),
        "PropertyRowDemo" => UiComponentState::new()
            .with_value("value", UiValue::String("Label + Field".to_string())),
        "GroupDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(true)),
        "FoldoutDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(false)),
        "InspectorSectionDemo" => {
            UiComponentState::new().with_value("expanded", UiValue::Bool(true))
        }
        "ArrayFieldDemo" => UiComponentState::new().with_value(
            "items",
            UiValue::Array(vec![
                UiValue::String("Label".to_string()),
                UiValue::String("NumberField".to_string()),
                UiValue::String("AssetField".to_string()),
            ]),
        ),
        "MapFieldDemo" => {
            let mut entries = BTreeMap::new();
            entries.insert("speed".to_string(), UiValue::Float(1.0));
            entries.insert("visible".to_string(), UiValue::Bool(true));
            UiComponentState::new().with_value("entries", UiValue::Map(entries))
        }
        "ToggleButtonDemo" | "CheckboxDemo" => {
            UiComponentState::new().with_value("value", UiValue::Bool(true))
        }
        "RadioDemo" => UiComponentState::new().with_value("value", UiValue::Bool(false)),
        "ListRowDemo" => {
            UiComponentState::new().with_value("value", UiValue::String("selected".to_string()))
        }
        "TreeRowDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(true)),
        "ContextActionMenuDemo" => {
            UiComponentState::new().with_value("value", UiValue::String("Inspect".to_string()))
        }
        _ => UiComponentState::new(),
    }
}
