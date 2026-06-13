use std::collections::BTreeSet;

use crate::input::{InputActionMap, InputActionState, InputButton, InputFrameSnapshot};

#[derive(Clone, Debug, Default)]
pub struct InputActionEvaluator {
    action_map: InputActionMap,
}

impl InputActionEvaluator {
    pub fn new(action_map: InputActionMap) -> Self {
        Self { action_map }
    }

    pub fn action_map(&self) -> &InputActionMap {
        &self.action_map
    }

    pub fn set_action_map(&mut self, action_map: InputActionMap) {
        self.action_map = action_map;
    }

    pub fn evaluate(&self, frame: &InputFrameSnapshot) -> InputActionState {
        self.evaluate_with_consumed_buttons(frame, &[])
    }

    pub fn evaluate_with_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        let consumed_buttons = consumed_buttons.iter().cloned().collect::<BTreeSet<_>>();
        let mut pressed = BTreeSet::new();
        let mut just_activated = BTreeSet::new();
        let mut just_deactivated = BTreeSet::new();

        for action in &self.action_map.actions {
            let mut action_pressed = false;
            let mut action_just_activated = false;
            let mut action_just_deactivated = false;

            for binding in self.action_map.bindings_for_action(&action.id) {
                if binding
                    .buttons
                    .iter()
                    .any(|button| consumed_buttons.contains(button))
                {
                    continue;
                }

                let all_pressed = binding
                    .buttons
                    .iter()
                    .all(|button| frame.buttons.pressed(button));
                let any_just_pressed = binding
                    .buttons
                    .iter()
                    .any(|button| frame.buttons.just_pressed(button));
                let any_just_released = binding
                    .buttons
                    .iter()
                    .any(|button| frame.buttons.just_released(button));

                if all_pressed {
                    action_pressed = true;
                    action_just_activated |= any_just_pressed;
                } else {
                    action_just_deactivated |= any_just_released;
                }
            }

            if action_pressed {
                pressed.insert(action.id.clone());
                if action_just_activated {
                    just_activated.insert(action.id.clone());
                }
            } else if action_just_deactivated {
                just_deactivated.insert(action.id.clone());
            }
        }

        InputActionState::from_sets(pressed, just_activated, just_deactivated)
    }
}
