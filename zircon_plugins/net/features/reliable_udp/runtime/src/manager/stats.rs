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
