use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::super::pane_value_conversion::value_as_string;
use super::super::showcase_actions::showcase_action_id_for_suffix;
use super::empty::empty_collection_field;
use super::roles::{collection_field_checked, collection_field_role};
use super::type_tokens::CollectionTypeTraits;
use super::validation::collection_map_entry_validation;

pub(super) fn map_collection_fields(
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[RetainedUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneCollectionFieldData> {
    let key_type = attributes
        .get("key_type")
        .and_then(value_as_string)
        .unwrap_or_else(|| "Key".to_string());
    let value_type = attributes
        .get("value_type")
        .and_then(value_as_string)
        .unwrap_or_else(|| "Value".to_string());
    let edit_action_id = showcase_action_id_for_suffix(bindings, "MapFieldSetEntry");
    let remove_action_id = showcase_action_id_for_suffix(bindings, "MapFieldRemoveEntry");
    let entries = attributes.get("entries").map(UiValue::from_toml);
    let Some(UiValue::Map(values)) = entries else {
        return vec![empty_collection_field(
            "map-empty",
            key_type.as_str(),
            "",
            value_type.as_str(),
            format!("Empty {key_type} -> {value_type} map"),
        )];
    };
    if values.is_empty() {
        return vec![empty_collection_field(
            "map-empty",
            key_type.as_str(),
            "",
            value_type.as_str(),
            format!("Empty {key_type} -> {value_type} map"),
        )];
    }
    let key_traits = CollectionTypeTraits::from_declared_type(&key_type);
    let value_traits = CollectionTypeTraits::from_declared_type(&value_type);
    values
        .into_iter()
        .map(|(key, value)| {
            let validation = collection_map_entry_validation(
                &key_type,
                key_traits,
                &key,
                &value_type,
                value_traits,
                &value,
            );
            host_contract::TemplatePaneCollectionFieldData {
                row_id: format!("map-{key}").into(),
                index_text: "".into(),
                key_type: key_type.clone().into(),
                key_component_role: collection_field_role(key_traits, None).into(),
                key_text: key.into(),
                value_type: value_type.clone().into(),
                value_component_role: collection_field_role(value_traits, Some(&value)).into(),
                value_text: value.display_text().into(),
                value_checked: collection_field_checked(&value),
                validation_level: validation.level.into(),
                validation_message: validation.message.into(),
                key_edit_action_id: edit_action_id.clone().into(),
                edit_action_id: edit_action_id.clone().into(),
                remove_action_id: remove_action_id.clone().into(),
                move_up_action_id: "".into(),
                move_up_payload: "".into(),
                move_down_action_id: "".into(),
                move_down_payload: "".into(),
                empty: false,
            }
        })
        .collect()
}
