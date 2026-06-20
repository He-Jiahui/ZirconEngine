use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct TemplatePaneCollectionFieldData {
    pub row_id: SharedString,
    pub index_text: SharedString,
    pub key_type: SharedString,
    pub key_component_role: SharedString,
    pub key_text: SharedString,
    pub value_type: SharedString,
    pub value_component_role: SharedString,
    pub value_text: SharedString,
    pub value_checked: bool,
    pub validation_level: SharedString,
    pub validation_message: SharedString,
    pub key_edit_action_id: SharedString,
    pub edit_action_id: SharedString,
    pub remove_action_id: SharedString,
    pub move_up_action_id: SharedString,
    pub move_up_payload: SharedString,
    pub move_down_action_id: SharedString,
    pub move_down_payload: SharedString,
    pub empty: bool,
}
