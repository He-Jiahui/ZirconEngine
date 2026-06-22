use crate::ui::template_runtime::RetainedUiHostBindingProjection;

use super::binding_ids::{
    first_showcase_binding, showcase_action_id_for_binding_id, showcase_binding_with_suffix,
};

pub(in super::super) fn preferred_showcase_action_id(
    control_id: &str,
    popup_open: bool,
    bindings: &[RetainedUiHostBindingProjection],
) -> Option<String> {
    preferred_action_suffix(control_id, popup_open)
        .and_then(|suffix| showcase_binding_with_suffix(bindings, suffix))
        .or_else(|| first_showcase_binding(bindings))
        .map(|binding| showcase_action_id_for_binding_id(&binding.binding_id))
}

fn preferred_action_suffix(control_id: &str, popup_open: bool) -> Option<&'static str> {
    match control_id {
        "NumberFieldDemo" => Some("NumberFieldDragUpdate"),
        "RangeFieldDemo" => Some("RangeFieldChanged"),
        "SliderDemo" => Some("SliderChanged"),
        "RangeSliderDemo" => Some("RangeSliderChanged"),
        "ColorFieldDemo" => Some("ColorFieldChanged"),
        "Vector2FieldDemo" => Some("Vector2FieldChanged"),
        "Vector3FieldDemo" => Some("Vector3FieldChanged"),
        "Vector4FieldDemo" => Some("Vector4FieldChanged"),
        "TabDemo" => Some("TabChanged"),
        "TabStripDemo" => Some("TabStripChanged"),
        "DropdownDemo" => Some(if popup_open {
            "DropdownChanged"
        } else {
            "DropdownOpenPopup"
        }),
        "ComboBoxDemo" => Some(if popup_open {
            "ComboBoxChanged"
        } else {
            "ComboBoxOpenPopup"
        }),
        "EnumFieldDemo" => Some(if popup_open {
            "EnumFieldChanged"
        } else {
            "EnumFieldOpenPopup"
        }),
        "FlagsFieldDemo" => Some(if popup_open {
            "FlagsFieldChanged"
        } else {
            "FlagsFieldOpenPopup"
        }),
        "SearchSelectDemo" => Some(if popup_open {
            "SearchSelectChanged"
        } else {
            "SearchSelectOpenPopup"
        }),
        "AssetFieldDemo" => Some("AssetFieldDropped"),
        "InstanceFieldDemo" => Some("InstanceFieldDropped"),
        "ObjectFieldDemo" => Some("ObjectFieldDropped"),
        "GroupDemo" => Some("GroupToggled"),
        "FoldoutDemo" => Some("FoldoutToggled"),
        "InspectorSectionDemo" => Some("InspectorSectionToggled"),
        "ArrayFieldDemo" => Some("ArrayFieldAddElement"),
        "MapFieldDemo" => Some("MapFieldAddEntry"),
        "TreeRowDemo" => Some("TreeRowToggled"),
        "ContextActionMenuDemo" => Some(if popup_open {
            "ContextActionMenuChanged"
        } else {
            "ContextActionMenuOpenAt"
        }),
        _ => None,
    }
}
