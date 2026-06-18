use super::{GamepadAxisInput, InputActionMap, InputActionState, InputButton, InputFrameSnapshot};

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
    fn evaluate_actions_with_active_contexts_and_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState;
}
