//! Topic-based event distribution with explicit delivery policy.

mod admission;
mod close;
mod diagnostics;
mod frozen;
mod prune;
mod publish;
mod subscribe;
mod subscriber;
mod topic;

use std::fmt;
use std::sync::Arc;

use crate::core::framework::events::{EventBusDiagnosticsMode, EventBusLimits};

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
    /// 按指定模式初始化总线状态；默认构造使用框架默认诊断模式，显式构造可选全量、采样或关闭诊断。
    pub fn new(diagnostics_mode: EventBusDiagnosticsMode) -> Self {
        Self {
            state: Arc::new(EventBusState::new(
                diagnostics_mode,
                EventBusLimits::default(),
            )),
        }
    }
    pub fn with_limits(diagnostics_mode: EventBusDiagnosticsMode, limits: EventBusLimits) -> Self {
        Self {
            state: Arc::new(EventBusState::new(diagnostics_mode, limits)),
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
