use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};

use super::menu_action_from_id::menu_action_from_id;
use super::{EditorHostEvent, EditorHostEventError};

/// 将菜单载荷还原为宿主领域事件，供事件正常化和保留界面菜单回调共用。
/// 调用方先按载荷域分流，并另行检查命令可用性；此入口不执行动作，也不证明界面来源可信。
pub fn dispatch_editor_host_binding(
    binding: &EditorUiBinding,
) -> Result<EditorHostEvent, EditorHostEventError> {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { action_id } => menu_action_from_id(action_id)
            .map(EditorHostEvent::Menu)
            .ok_or_else(|| EditorHostEventError::UnknownMenuAction(action_id.clone())),
        _ => Err(EditorHostEventError::UnsupportedPayload),
    }
}
