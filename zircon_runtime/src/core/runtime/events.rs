//! Topic-based event distribution with explicit delivery policy.

mod diagnostics;
mod prune;
mod publish;
mod subscribe;
mod subscriber;
mod topic;

use std::fmt;
use std::sync::Arc;

use crate::core::framework::events::EventBusDiagnosticsMode;

use topic::EventBusState;

#[derive(Clone)]
pub struct EventBus {
    state: Arc<EventBusState>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(EventBusDiagnosticsMode::Enabled)
    }
}

impl EventBus {
    pub fn new(diagnostics_mode: EventBusDiagnosticsMode) -> Self {
        Self {
            state: Arc::new(EventBusState::new(diagnostics_mode)),
        }
    }
}

impl fmt::Debug for EventBus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventBus")
            .field("diagnostics", &self.diagnostic_report())
            .finish()
    }
}
