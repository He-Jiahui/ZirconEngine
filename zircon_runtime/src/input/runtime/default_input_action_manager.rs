use std::sync::{Mutex, MutexGuard};

use crate::core::framework::input::InputActionManager;
use crate::input::{
    GamepadAxisInput, InputActionMap, InputActionState, InputButton, InputFrameSnapshot,
};

use super::InputActionEvaluator;

/// 注册表中的动作服务适配层；外层锁串行化配置替换与求值，并借用同一工作区避免重复加锁。
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

    fn lock_evaluator(&self) -> MutexGuard<'_, InputActionEvaluator> {
        self.evaluator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl InputActionManager for DefaultInputActionManager {
    fn action_map(&self) -> InputActionMap {
        self.lock_evaluator().action_map().clone()
    }

    fn set_action_map(&self, action_map: InputActionMap) {
        self.lock_evaluator().set_action_map(action_map);
    }

    fn evaluate_actions(&self, frame: &InputFrameSnapshot) -> InputActionState {
        self.lock_evaluator()
            .evaluate_while_manager_locked(frame, &[], &[], &[])
    }

    fn evaluate_actions_with_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.lock_evaluator()
            .evaluate_while_manager_locked(frame, &[], consumed_buttons, &[])
    }

    fn evaluate_actions_with_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        self.lock_evaluator().evaluate_while_manager_locked(
            frame,
            &[],
            consumed_buttons,
            consumed_axes,
        )
    }

    fn evaluate_actions_with_active_contexts(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
    ) -> InputActionState {
        self.lock_evaluator()
            .evaluate_while_manager_locked(frame, active_contexts, &[], &[])
    }

    fn evaluate_actions_with_active_contexts_and_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.lock_evaluator().evaluate_while_manager_locked(
            frame,
            active_contexts,
            consumed_buttons,
            &[],
        )
    }

    fn evaluate_actions_with_active_contexts_and_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        self.lock_evaluator().evaluate_while_manager_locked(
            frame,
            active_contexts,
            consumed_buttons,
            consumed_axes,
        )
    }
}

#[cfg(test)]
#[path = "tests/default_input_action_manager.rs"]
mod tests;
