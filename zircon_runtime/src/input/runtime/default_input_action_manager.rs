use std::sync::{Mutex, MutexGuard};

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
        self.lock_evaluator().evaluate(frame)
    }

    fn evaluate_actions_with_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.lock_evaluator()
            .evaluate_with_consumed_buttons(frame, consumed_buttons)
    }

    fn evaluate_actions_with_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        self.lock_evaluator()
            .evaluate_with_consumed_input(frame, consumed_buttons, consumed_axes)
    }

    fn evaluate_actions_with_active_contexts(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
    ) -> InputActionState {
        self.lock_evaluator()
            .evaluate_with_active_contexts(frame, active_contexts)
    }

    fn evaluate_actions_with_active_contexts_and_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.lock_evaluator()
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
        self.lock_evaluator()
            .evaluate_with_active_contexts_and_consumed_input(
                frame,
                active_contexts,
                consumed_buttons,
                consumed_axes,
            )
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use crate::core::framework::input::InputActionManager;
    use crate::input::{InputAction, InputActionMap, InputFrameSnapshot};

    use super::DefaultInputActionManager;

    #[test]
    fn input_action_manager_accessors_recover_poisoned_evaluator_lock() {
        let manager = DefaultInputActionManager::default();
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.lock_evaluator();
            panic!("poison input action evaluator");
        }));

        let mut action_map = InputActionMap::new();
        action_map.add_action(InputAction::new("gameplay.jump"));
        manager.set_action_map(action_map.clone());

        assert_eq!(manager.action_map(), action_map);
        assert!(!manager
            .evaluate_actions(&InputFrameSnapshot::default())
            .pressed("gameplay.jump"));
    }
}
