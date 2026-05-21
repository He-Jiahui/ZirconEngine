use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowExitCondition {
    OnPrimaryClosed,
    #[default]
    OnAllClosed,
    DontExit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowLifecyclePolicy {
    pub exit_condition: WindowExitCondition,
    pub close_when_requested: bool,
}

impl WindowLifecyclePolicy {
    pub fn with_exit_condition(mut self, exit_condition: WindowExitCondition) -> Self {
        self.exit_condition = exit_condition;
        self
    }

    pub fn with_close_when_requested(mut self, close_when_requested: bool) -> Self {
        self.close_when_requested = close_when_requested;
        self
    }

    pub fn should_close_on_request(self) -> bool {
        self.close_when_requested
    }

    pub fn should_exit_after_primary_close(self) -> bool {
        self.close_when_requested
            && matches!(
                self.exit_condition,
                WindowExitCondition::OnPrimaryClosed | WindowExitCondition::OnAllClosed
            )
    }

    pub fn diagnostic_lines(self) -> [String; 2] {
        [
            format!("window.exit_condition={:?}", self.exit_condition),
            format!("window.close_when_requested={}", self.close_when_requested),
        ]
    }
}

impl Default for WindowLifecyclePolicy {
    fn default() -> Self {
        Self {
            exit_condition: WindowExitCondition::OnAllClosed,
            close_when_requested: true,
        }
    }
}
