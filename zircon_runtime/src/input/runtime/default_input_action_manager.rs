use std::sync::Mutex;

use crate::core::framework::input::InputActionManager;
use crate::input::{
    GamepadAxisInput, InputActionMap, InputActionState, InputButton, InputFrameSnapshot,
};

use super::InputActionEvaluator;

#[derive(Debug, Default)]
pub struct DefaultInputActionManager {
    evaluator: Mutex<InputActionEvaluator>,
}

impl DefaultInputActionManager {
    pub fn new(action_map: InputActionMap) -> Self {
        Self {
            evaluator: Mutex::new(InputActionEvaluator::new(action_map)),
        }
    }
}

impl InputActionManager for DefaultInputActionManager {
    fn action_map(&self) -> InputActionMap {
        self.evaluator.lock().unwrap().action_map().clone()
    }

    fn set_action_map(&self, action_map: InputActionMap) {
        self.evaluator.lock().unwrap().set_action_map(action_map);
    }

    fn evaluate_actions(&self, frame: &InputFrameSnapshot) -> InputActionState {
        self.evaluator.lock().unwrap().evaluate(frame)
    }

    fn evaluate_actions_with_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.evaluator
            .lock()
            .unwrap()
            .evaluate_with_consumed_buttons(frame, consumed_buttons)
    }

    fn evaluate_actions_with_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        self.evaluator.lock().unwrap().evaluate_with_consumed_input(
            frame,
            consumed_buttons,
            consumed_axes,
        )
    }

    fn evaluate_actions_with_active_contexts(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
    ) -> InputActionState {
        self.evaluator
            .lock()
            .unwrap()
            .evaluate_with_active_contexts(frame, active_contexts)
    }

    fn evaluate_actions_with_active_contexts_and_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.evaluator
            .lock()
            .unwrap()
            .evaluate_with_active_contexts_and_consumed_buttons(
                frame,
                active_contexts,
                consumed_buttons,
            )
    }

    fn evaluate_actions_with_active_contexts_and_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        self.evaluator
            .lock()
            .unwrap()
            .evaluate_with_active_contexts_and_consumed_input(
                frame,
                active_contexts,
                consumed_buttons,
                consumed_axes,
            )
    }
}
