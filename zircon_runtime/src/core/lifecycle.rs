//! Service lifecycle and startup mode.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupMode {
    Immediate,
    Lazy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Registered,
    Initializing,
    Running,
    Stopping,
    Unloaded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceKind {
    Driver,
    Manager,
    Plugin,
}

impl ServiceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Driver => "Driver",
            Self::Manager => "Manager",
            Self::Plugin => "Plugin",
        }
    }
}
