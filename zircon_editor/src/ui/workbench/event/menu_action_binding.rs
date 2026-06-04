use crate::core::editor_event::MenuAction;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::constants::WORKBENCH_MENU_VIEW_ID;
use super::menu_action_id::{menu_action_control_id, menu_action_id};

pub fn menu_action_binding(action: &MenuAction) -> EditorUiBinding {
    let action_id = menu_action_id(action);
    let control_id = menu_action_control_id(action);
    EditorUiBinding::new(
        WORKBENCH_MENU_VIEW_ID,
        control_id,
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action(action_id),
    )
}
