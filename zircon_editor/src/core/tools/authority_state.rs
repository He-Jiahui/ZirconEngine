use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolAuthorityState {
    #[default]
    Open,
    Quiescing,
    Draining,
    Faulted,
    Closed,
}

impl ToolAuthorityState {
    pub const fn accepts_requests(self) -> bool {
        matches!(self, Self::Open)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToolShutdownOutcome {
    pub(crate) released_single_leases: usize,
    pub(crate) released_set_leases: usize,
    pub(crate) withdrawn_single_requests: usize,
    pub(crate) withdrawn_set_requests: usize,
}

impl ToolShutdownOutcome {
    pub const fn released_single_leases(self) -> usize {
        self.released_single_leases
    }

    pub const fn released_set_leases(self) -> usize {
        self.released_set_leases
    }

    pub const fn withdrawn_single_requests(self) -> usize {
        self.withdrawn_single_requests
    }

    pub const fn withdrawn_set_requests(self) -> usize {
        self.withdrawn_set_requests
    }
}
