use crate::core::editor_operation::EditorOperationPath;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::constants::WORKBENCH_MENU_VIEW_ID;

pub fn editor_operation_binding(operation: &EditorOperationPath) -> EditorUiBinding {
    EditorUiBinding::new(
        WORKBENCH_MENU_VIEW_ID,
        operation.as_str(),
        EditorUiEventKind::Click,
        EditorUiBindingPayload::editor_operation(operation.as_str()),
    )
}
