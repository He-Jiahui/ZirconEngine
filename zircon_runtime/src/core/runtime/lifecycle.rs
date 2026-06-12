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
    pub fn from_registry_segment(value: &str) -> Option<Self> {
        Self::from_registry_segment_bytes(value.as_bytes())
    }

    pub(crate) fn from_registry_segment_bytes(value: &[u8]) -> Option<Self> {
        match value {
            b"Driver" => Some(Self::Driver),
            b"Manager" => Some(Self::Manager),
            b"Plugin" => Some(Self::Plugin),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Driver => "Driver",
            Self::Manager => "Manager",
            Self::Plugin => "Plugin",
        }
    }
}
