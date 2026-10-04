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
