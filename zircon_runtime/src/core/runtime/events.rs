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

/// CoreRuntime 的主题事件总线句柄；克隆后共享订阅者、投递策略和诊断状态。
///
/// 订阅所有权在返回的 subscription，持有者负责消费并在不再需要时释放它。
#[derive(Clone)]
pub struct EventBus {
    state: Arc<EventBusState>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(EventBusDiagnosticsMode::default())
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
