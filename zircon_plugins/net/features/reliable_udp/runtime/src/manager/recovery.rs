//! 公开连接、恢复与断开诊断快照，供重发上限及模拟丢失阈值改变恢复状态。
//! 状态变化本身不会触发 socket 重连或向根 NetManager 发布事件。

use zircon_runtime::core::framework::net::{
    ReliableDatagramRecoveryReport, ReliableDatagramRecoveryState,
};

use super::NetReliableUdpRuntimeManager;

impl NetReliableUdpRuntimeManager {
    pub(in crate::manager) fn recovery_state_impl(&self) -> ReliableDatagramRecoveryReport {
        self.state
            .lock()
            .expect("net reliable UDP state mutex poisoned")
            .recovery_report()
    }

    pub(in crate::manager) fn mark_disconnected_impl(
        &self,
        diagnostic: String,
    ) -> ReliableDatagramRecoveryReport {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        state.recovery_state = ReliableDatagramRecoveryState::Disconnected;
        state.recovery_diagnostic = Some(diagnostic);
        state.recovery_report()
    }

    pub(in crate::manager) fn mark_recovered_impl(&self) -> ReliableDatagramRecoveryReport {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        state.recovery_state = ReliableDatagramRecoveryState::Connected;
        state.dropped_packets_since_recovery = 0;
        state.recovery_diagnostic = None;
        state.recovery_report()
    }
}
