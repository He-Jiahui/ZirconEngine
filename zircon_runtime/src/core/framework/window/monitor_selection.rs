use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMonitorSelection {
    Current,
    #[default]
    Primary,
    Index(usize),
}

impl WindowMonitorSelection {
    pub const fn index(index: usize) -> Self {
        Self::Index(index)
    }
}
