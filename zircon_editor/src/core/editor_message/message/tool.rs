use serde::{Deserialize, Serialize};

use crate::core::tools::ToolLifecycleEvent;

/// Typed lifecycle fact emitted by the sole ToolScheduler service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolMessage {
    Lifecycle(ToolLifecycleEvent),
}
