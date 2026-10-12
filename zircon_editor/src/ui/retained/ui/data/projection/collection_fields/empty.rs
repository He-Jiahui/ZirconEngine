use crate::ui::retained_host as host_contract;

use super::roles::collection_field_role;
use super::type_tokens::CollectionTypeTraits;

pub(super) fn empty_collection_field(
    row_id: &str,
    key_type: &str,
    key_text: &str,
    value_type: &str,
    message: String,
) -> host_contract::TemplatePaneCollectionFieldData {
    host_contract::TemplatePaneCollectionFieldData {
        row_id: row_id.into(),
        index_text: "".into(),
        key_type: key_type.into(),
        key_component_role: collection_field_role(
            CollectionTypeTraits::from_declared_type(key_type),
            None,
        )
        .into(),
        key_text: key_text.into(),
        value_type: value_type.into(),
        value_component_role: collection_field_role(
            CollectionTypeTraits::from_declared_type(value_type),
            None,
        )
        .into(),
        value_text: "".into(),
        value_checked: false,
        validation_level: "warning".into(),
        validation_message: message.into(),
        key_edit_action_id: "".into(),
        edit_action_id: "".into(),
        remove_action_id: "".into(),
        move_up_action_id: "".into(),
        move_up_payload: "".into(),
        move_down_action_id: "".into(),
        move_down_payload: "".into(),
        empty: true,
    }
}
