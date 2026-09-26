use super::{GamepadAxisInput, InputActionMap, InputActionState, InputButton, InputFrameSnapshot};

/// 将物理帧快照求值为命名动作的服务边界；映射修改与求值由实现保持一致视图。
/// 调用方可先按 UI 或更高层路由结果标记已消费按钮、轴，再求值剩余输入。
pub trait InputActionManager: Send + Sync {
    fn action_map(&self) -> InputActionMap;
    fn set_action_map(&self, action_map: InputActionMap);
    fn evaluate_actions(&self, frame: &InputFrameSnapshot) -> InputActionState;
    fn evaluate_actions_with_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
    ) -> InputActionState;
    fn evaluate_actions_with_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState;
    fn evaluate_actions_with_active_contexts(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
    ) -> InputActionState;
    fn evaluate_actions_with_active_contexts_and_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
    ) -> InputActionState;
    /// 同时限定本次可用上下文和已消费的物理输入；空上下文列表表示使用全部启用上下文。
    fn evaluate_actions_with_active_contexts_and_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState;
}
