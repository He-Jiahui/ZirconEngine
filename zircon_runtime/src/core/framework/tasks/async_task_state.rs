use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AsyncTaskState {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl AsyncTaskState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}
