use serde::{Deserialize, Serialize};

use crate::core::tools::ToolTransitionBatch;

/// Ordered transition emitted by the sole ToolScheduler service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolMessage {
    Transition(ToolTransitionBatch),
}
