use crate::ui::slint_host as host_contract;
use crate::ui::template_runtime::SlintUiHostBindingProjection;

pub(super) fn showcase_binding_id_for_suffix(
    bindings: &[SlintUiHostBindingProjection],
    suffix: &str,
) -> String {
    bindings
        .iter()
        .find(|binding| {
            binding.binding_id.starts_with("UiComponentShowcase/")
                && binding.binding_id.ends_with(suffix)
        })
        .map(|binding| binding.binding_id.clone())
        .unwrap_or_default()
}

pub(super) fn preferred_showcase_action_id(
    control_id: &str,
    popup_open: bool,
    bindings: &[SlintUiHostBindingProjection],
) -> Option<String> {
    let preferred = match control_id {
        "NumberFieldDemo" => Some("NumberFieldDragUpdate"),
        "RangeFieldDemo" => Some("RangeFieldChanged"),
        "ColorFieldDemo" => Some("ColorFieldChanged"),
        "Vector2FieldDemo" => Some("Vector2FieldChanged"),
        "Vector3FieldDemo" => Some("Vector3FieldChanged"),
        "Vector4FieldDemo" => Some("Vector4FieldChanged"),
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
    };
    preferred
        .and_then(|suffix| {
            bindings.iter().find(|binding| {
                binding.binding_id.starts_with("UiComponentShowcase/")
                    && binding.binding_id.ends_with(suffix)
            })
        })
        .or_else(|| {
            bindings
                .iter()
                .find(|binding| binding.binding_id.starts_with("UiComponentShowcase/"))
        })
        .map(|binding| binding.binding_id.clone())
}

pub(super) fn preferred_showcase_drag_action_id(
    control_id: &str,
    bindings: &[SlintUiHostBindingProjection],
) -> Option<String> {
    let suffix = match control_id {
        "NumberFieldDemo" => Some("NumberFieldDragUpdate"),
        "RangeFieldDemo" => Some("RangeFieldDragUpdate"),
        _ => None,
    }?;
    bindings
        .iter()
        .find(|binding| {
            binding.binding_id.starts_with("UiComponentShowcase/")
                && binding.binding_id.ends_with(suffix)
        })
        .map(|binding| binding.binding_id.clone())
}

pub(super) fn preferred_showcase_pointer_drag_action_id(
    control_id: &str,
    event_suffix: &str,
    bindings: &[SlintUiHostBindingProjection],
) -> Option<String> {
    let suffix = match (control_id, event_suffix) {
        ("NumberFieldDemo", "DragBegin") => Some("NumberFieldDragBegin"),
        ("NumberFieldDemo", "DragEnd") => Some("NumberFieldDragEnd"),
        _ => None,
    }?;
    bindings
        .iter()
        .find(|binding| {
            binding.binding_id.starts_with("UiComponentShowcase/")
                && binding.binding_id.ends_with(suffix)
        })
        .map(|binding| binding.binding_id.clone())
}

pub(super) fn preferred_showcase_edit_action_id(
    control_id: &str,
    bindings: &[SlintUiHostBindingProjection],
) -> Option<String> {
    let suffix = match control_id {
        "InputFieldDemo" => Some("InputFieldChanged"),
        "TextFieldDemo" => Some("TextFieldChanged"),
        "NumberFieldDemo" => Some("NumberFieldChanged"),
        "RangeFieldDemo" => Some("RangeFieldChanged"),
        "SearchSelectDemo" => Some("SearchSelectQueryChanged"),
        _ => None,
    }?;
    bindings
        .iter()
        .find(|binding| {
            binding.binding_id.starts_with("UiComponentShowcase/")
                && binding.binding_id.ends_with(suffix)
        })
        .map(|binding| binding.binding_id.clone())
}

pub(super) fn preferred_showcase_commit_action_id(
    control_id: &str,
    bindings: &[SlintUiHostBindingProjection],
) -> Option<String> {
    let suffix = match control_id {
        "InputFieldDemo" => Some("InputFieldCommitted"),
        "TextFieldDemo" => Some("TextFieldCommitted"),
        "NumberFieldDemo" => Some("NumberFieldCommitted"),
        "RangeFieldDemo" => Some("RangeFieldCommitted"),
        _ => None,
    }?;
    bindings
        .iter()
        .find(|binding| {
            binding.binding_id.starts_with("UiComponentShowcase/")
                && binding.binding_id.ends_with(suffix)
        })
        .map(|binding| binding.binding_id.clone())
}

pub(super) fn preferred_showcase_action_buttons(
    control_id: &str,
    bindings: &[SlintUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneActionData> {
    let actions: &[(&str, &str)] = match control_id {
        "AssetFieldDemo" => &[
            ("Find", "AssetFieldLocate"),
            ("Open", "AssetFieldOpen"),
            ("Clear", "AssetFieldClear"),
        ],
        "InstanceFieldDemo" => &[
            ("Find", "InstanceFieldLocate"),
            ("Open", "InstanceFieldOpen"),
            ("Clear", "InstanceFieldClear"),
        ],
        "ObjectFieldDemo" => &[
            ("Find", "ObjectFieldLocate"),
            ("Open", "ObjectFieldOpen"),
            ("Clear", "ObjectFieldClear"),
        ],
        "ArrayFieldDemo" => &[
            ("Add", "ArrayFieldAddElement"),
            ("Set", "ArrayFieldSetElement"),
            ("Remove", "ArrayFieldRemoveElement"),
            ("Move", "ArrayFieldMoveElement"),
        ],
        "MapFieldDemo" => &[
            ("Add", "MapFieldAddEntry"),
            ("Set", "MapFieldSetEntry"),
            ("Remove", "MapFieldRemoveEntry"),
        ],
        _ => &[],
    };
    actions
        .iter()
        .filter_map(|(label, suffix)| {
            bindings
                .iter()
                .find(|binding| {
                    binding.binding_id.starts_with("UiComponentShowcase/")
                        && binding.binding_id.ends_with(suffix)
                })
                .map(|binding| host_contract::TemplatePaneActionData {
                    label: (*label).into(),
                    action_id: binding.binding_id.clone().into(),
                })
        })
        .collect()
}
