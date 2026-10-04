//! 暴露可靠 UDP 的投递/重发/丢失统计并更新恢复阈值；pending_packets 是在途队列快照。
//! 统计由本 manager 的显式 API 维护，不自动反映根 UDP socket 的真实收发。

use zircon_runtime::core::framework::net::{ReliableDatagramPacket, ReliableDatagramStats};

use super::NetReliableUdpRuntimeManager;

impl NetReliableUdpRuntimeManager {
    pub(in crate::manager) fn record_dropped_packet_impl(&self) {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        state.stats.dropped_packets += 1;
        state.dropped_packets_since_recovery += 1;
        state.update_recovery_after_delivery();
    }

    pub(in crate::manager) fn record_rtt_ms_impl(&self, rtt_ms: f32) {
        self.state
            .lock()
            .expect("net reliable UDP state mutex poisoned")
            .stats
            .rtt_ms = rtt_ms;
    }

    pub(in crate::manager) fn pending_packets_impl(&self) -> Vec<ReliableDatagramPacket> {
        self.state
            .lock()
            .expect("net reliable UDP state mutex poisoned")
            .outbound
            .iter()
            .cloned()
            .collect()
    }

    pub(in crate::manager) fn stats_impl(&self) -> ReliableDatagramStats {
        self.state
            .lock()
            .expect("net reliable UDP state mutex poisoned")
            .stats
            .clone()
    }
}
