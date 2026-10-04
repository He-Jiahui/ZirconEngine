use crate::core::editor_operation::EditorOperationPath;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::constants::WORKBENCH_MENU_VIEW_ID;

/// 把已解析的操作身份交给菜单绑定；是否已注册、当前可执行以及调用来源由宿主操作入口核验。
pub fn editor_operation_binding(operation: &EditorOperationPath) -> EditorUiBinding {
    EditorUiBinding::new(
        WORKBENCH_MENU_VIEW_ID,
        operation.as_str(),
        EditorUiEventKind::Click,
        EditorUiBindingPayload::editor_operation(operation.as_str()),
    )
}
