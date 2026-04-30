use toml::Value as TomlValue;

use super::super::host_nodes::SlintUiHostNodeProjection;

const CATEGORY_ALL: &str = "All";
const CATEGORY_VISUAL: &str = "Visual";
const CATEGORY_INPUT: &str = "Input";
const CATEGORY_NUMERIC: &str = "Numeric";
const CATEGORY_SELECTION: &str = "Selection";
const CATEGORY_REFERENCE: &str = "Reference";
const CATEGORY_COLLECTIONS: &str = "Collections";
const CATEGORY_FEEDBACK: &str = "Feedback";

pub(super) fn project_selected_category_state(
    nodes: &mut [SlintUiHostNodeProjection],
    selected_category: &str,
) {
    for node in nodes {
        let Some(control_id) = node.control_id.as_deref() else {
            continue;
        };
        let Some(category) = nav_category_for_control(control_id) else {
            continue;
        };
        let selected = category == selected_category;
        node.attributes
            .insert("selected".to_string(), TomlValue::Boolean(selected));
        node.attributes.insert(
            "selection_state".to_string(),
            TomlValue::String(if selected { "selected" } else { "normal" }.to_string()),
        );
    }
}

pub(super) fn should_keep_for_selected_category(
    node: &SlintUiHostNodeProjection,
    selected_category: &str,
) -> bool {
    let Some(control_id) = node.control_id.as_deref() else {
        return true;
    };
    let Some(category) = demo_category_for_control(control_id) else {
        return true;
    };
    selected_category == CATEGORY_ALL || category == selected_category
}

fn nav_category_for_control(control_id: &str) -> Option<&'static str> {
    match control_id {
        "ShowAllCategory" => Some(CATEGORY_ALL),
        "ShowVisualCategory" => Some(CATEGORY_VISUAL),
        "ShowInputCategory" => Some(CATEGORY_INPUT),
        "ShowNumericCategory" => Some(CATEGORY_NUMERIC),
        "ShowSelectionCategory" => Some(CATEGORY_SELECTION),
        "ShowReferenceCategory" => Some(CATEGORY_REFERENCE),
        "ShowDataCategory" => Some(CATEGORY_COLLECTIONS),
        "ShowFeedbackCategory" => Some(CATEGORY_FEEDBACK),
        _ => None,
    }
}

fn demo_category_for_control(control_id: &str) -> Option<&'static str> {
    match control_id {
        "LabelDemo" | "RichLabelDemo" | "ImageDemo" | "IconDemo" | "SvgIconDemo"
        | "SeparatorDemo" => Some(CATEGORY_VISUAL),
        "ProgressBarDemo" | "SpinnerDemo" | "BadgeDemo" | "HelpRowDemo" => Some(CATEGORY_FEEDBACK),
        "ButtonDemo"
        | "IconButtonDemo"
        | "ToggleButtonDemo"
        | "CheckboxDemo"
        | "RadioDemo"
        | "SegmentedControlDemo"
        | "InputFieldDemo"
        | "TextFieldDemo" => Some(CATEGORY_INPUT),
        "NumberFieldDemo" | "RangeFieldDemo" | "ColorFieldDemo" | "Vector2FieldDemo"
        | "Vector3FieldDemo" | "Vector4FieldDemo" => Some(CATEGORY_NUMERIC),
        "DropdownDemo" | "ComboBoxDemo" | "EnumFieldDemo" | "FlagsFieldDemo"
        | "SearchSelectDemo" => Some(CATEGORY_SELECTION),
        "AssetFieldDemo" | "InstanceFieldDemo" | "ObjectFieldDemo" => Some(CATEGORY_REFERENCE),
        "GroupDemo"
        | "FoldoutDemo"
        | "PropertyRowDemo"
        | "InspectorSectionDemo"
        | "ArrayFieldDemo"
        | "MapFieldDemo"
        | "ListRowDemo"
        | "TreeRowDemo"
        | "ContextActionMenuDemo" => Some(CATEGORY_COLLECTIONS),
        _ => None,
    }
}
