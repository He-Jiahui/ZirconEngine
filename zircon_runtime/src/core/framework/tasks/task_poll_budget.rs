use serde::{Deserialize, Serialize};

pub const DEFAULT_MAIN_THREAD_POLLS_PER_FRAME: u32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskPollBudget {
    pub max_main_thread_polls_per_frame: Option<u32>,
}

impl TaskPollBudget {
    pub const fn new(max_main_thread_polls_per_frame: u32) -> Self {
        Self {
            max_main_thread_polls_per_frame: Some(max_main_thread_polls_per_frame),
        }
    }

    pub const fn unlimited() -> Self {
        Self {
            max_main_thread_polls_per_frame: None,
        }
    }

    pub fn remaining_after(self, completed_polls: u32) -> Option<u32> {
        self.max_main_thread_polls_per_frame
            .map(|max_polls| max_polls.saturating_sub(completed_polls))
    }

    pub fn is_exhausted_after(self, completed_polls: u32) -> bool {
        self.remaining_after(completed_polls) == Some(0)
    }
}

impl Default for TaskPollBudget {
    fn default() -> Self {
        Self::new(DEFAULT_MAIN_THREAD_POLLS_PER_FRAME)
    }
}
