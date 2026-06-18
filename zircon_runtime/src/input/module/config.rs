use serde::{Deserialize, Serialize};

use crate::core::framework::input::InputActionMap;

use super::super::runtime::{DefaultInputActionManager, InputActionEvaluator};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub action_map: InputActionMap,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            action_map: InputActionMap::default(),
        }
    }
}

impl InputConfig {
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_action_map(mut self, action_map: InputActionMap) -> Self {
        self.action_map = action_map;
        self
    }

    pub fn effective_action_map(&self) -> InputActionMap {
        if self.enabled {
            self.action_map.clone()
        } else {
            InputActionMap::default()
        }
    }

    pub fn action_evaluator(&self) -> InputActionEvaluator {
        InputActionEvaluator::new(self.effective_action_map())
    }

    pub fn action_manager(&self) -> DefaultInputActionManager {
        DefaultInputActionManager::new(self.effective_action_map())
    }
}
