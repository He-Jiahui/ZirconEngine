//! A lifecycle callback may retire the logical World or its Schedule.
use std::fmt;

use serde::{Deserialize, Serialize};

/// Transportable cancellation cause; it carries no World or Schedule authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleTargetChange {
    World,
    Schedule,
}

impl fmt::Display for LifecycleTargetChange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::World => "World",
            Self::Schedule => "Schedule",
        })
    }
}
