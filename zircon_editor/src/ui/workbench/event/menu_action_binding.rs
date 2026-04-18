use crate::ui::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::super::model::MenuAction;
use super::constants::WORKBENCH_MENU_VIEW_ID;
use super::menu_action_id::menu_action_id;

pub fn menu_action_binding(action: &MenuAction) -> EditorUiBinding {
    let action_id = menu_action_id(action);
    EditorUiBinding::new(
        WORKBENCH_MENU_VIEW_ID,
        action_id.clone(),
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action(action_id),
    )
}
