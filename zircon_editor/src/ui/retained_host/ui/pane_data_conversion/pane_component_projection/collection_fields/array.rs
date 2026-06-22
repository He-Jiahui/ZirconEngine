use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::super::pane_value_conversion::value_as_string;
use super::super::showcase_actions::showcase_action_id_for_suffix;
use super::empty::empty_collection_field;
use super::roles::{collection_field_checked, collection_field_role};
use super::validation::collection_value_validation;

pub(super) fn array_collection_fields(
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[RetainedUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneCollectionFieldData> {
    let element_type = attributes
        .get("element_type")
        .and_then(value_as_string)
        .unwrap_or_else(|| "Element".to_string());
    let edit_action_id = showcase_action_id_for_suffix(bindings, "ArrayFieldSetElement");
    let remove_action_id = showcase_action_id_for_suffix(bindings, "ArrayFieldRemoveElement");
    let move_action_id = showcase_action_id_for_suffix(bindings, "ArrayFieldMoveElement");
    let items = attributes.get("items").map(UiValue::from_toml);
    let Some(UiValue::Array(values)) = items else {
        return vec![empty_collection_field(
            "array-empty",
            "",
            "",
            element_type.as_str(),
            format!("Empty {element_type} list"),
        )];
    };
    if values.is_empty() {
        return vec![empty_collection_field(
            "array-empty",
            "",
            "",
            element_type.as_str(),
            format!("Empty {element_type} list"),
        )];
    }
    let value_count = values.len();
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let validation = collection_value_validation(&element_type, &value, "array element");
            host_contract::TemplatePaneCollectionFieldData {
                row_id: format!("array-{index}").into(),
                index_text: format!("#{index}").into(),
                key_type: "".into(),
                key_component_role: "".into(),
                key_text: "".into(),
                value_type: element_type.clone().into(),
                value_component_role: collection_field_role(&element_type, Some(&value)).into(),
                value_text: value.display_text().into(),
                value_checked: collection_field_checked(&value),
                validation_level: validation.level.into(),
                validation_message: validation.message.into(),
                key_edit_action_id: "".into(),
                edit_action_id: edit_action_id.clone().into(),
                remove_action_id: remove_action_id.clone().into(),
                move_up_action_id: if index > 0 {
                    move_action_id.clone().into()
                } else {
                    "".into()
                },
                move_up_payload: if index > 0 {
                    format!("array-{index}={}", index - 1).into()
                } else {
                    "".into()
                },
                move_down_action_id: if index + 1 < value_count {
                    move_action_id.clone().into()
                } else {
                    "".into()
                },
                move_down_payload: if index + 1 < value_count {
                    format!("array-{index}={}", index + 1).into()
                } else {
                    "".into()
                },
                empty: false,
            }
        })
        .collect()
}
