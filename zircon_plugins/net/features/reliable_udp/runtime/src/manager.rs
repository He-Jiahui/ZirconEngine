use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramDeliveryReport,
    ReliableDatagramPacket, ReliableDatagramReceiveReport, ReliableDatagramRecoveryReport,
    ReliableDatagramSendReport, ReliableDatagramSimulationProfile, ReliableDatagramStats,
};

mod assembly;
mod delivery;
mod receive;
mod recovery;
mod resend;
mod send;
mod state;
mod stats;

pub(self) const RESEND_ATTEMPT_CAP_DIAGNOSTIC: &str =
    "reliable datagram resend attempt cap exceeded";

#[derive(Clone, Debug)]
pub struct NetReliableUdpRuntimeManager {
    pub(in crate::manager) state: Arc<Mutex<state::NetReliableUdpRuntimeState>>,
}

impl NetReliableUdpRuntimeManager {
    pub fn new(config: ReliableDatagramConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(state::NetReliableUdpRuntimeState::new(config))),
        }
    }

    pub fn set_simulation_profile(&self, profile: ReliableDatagramSimulationProfile) {
        self.set_simulation_profile_impl(profile);
    }

    pub fn enqueue_reliable_datagram(
        &self,
        channel: impl Into<String>,
        payload: impl Into<Vec<u8>>,
    ) -> ReliableDatagramSendReport {
        self.enqueue_reliable_datagram_impl(channel.into(), payload.into())
    }

    pub fn acknowledge(&self, ack: ReliableDatagramAck) -> usize {
        self.acknowledge_impl(ack)
    }

    pub fn simulate_outbound_delivery(
        &self,
        packets: impl IntoIterator<Item = ReliableDatagramPacket>,
    ) -> ReliableDatagramDeliveryReport {
        self.simulate_outbound_delivery_impl(packets)
    }

    pub fn receive_packet(&self, packet: ReliableDatagramPacket) -> ReliableDatagramReceiveReport {
        self.receive_packet_impl(packet)
    }

    pub fn recovery_state(&self) -> ReliableDatagramRecoveryReport {
        self.recovery_state_impl()
    }

    pub fn mark_disconnected(
        &self,
        diagnostic: impl Into<String>,
    ) -> ReliableDatagramRecoveryReport {
        self.mark_disconnected_impl(diagnostic.into())
    }

    pub fn mark_recovered(&self) -> ReliableDatagramRecoveryReport {
        self.mark_recovered_impl()
    }

    pub fn resend_pending(&self, max_packets: usize) -> Vec<ReliableDatagramPacket> {
        self.resend_pending_impl(max_packets)
    }

    pub fn resend_due(&self, now_ms: u64) -> Vec<ReliableDatagramPacket> {
        self.resend_due_impl(now_ms)
    }

    pub fn record_dropped_packet(&self) {
        self.record_dropped_packet_impl();
    }

    pub fn record_rtt_ms(&self, rtt_ms: f32) {
        self.record_rtt_ms_impl(rtt_ms);
    }

    pub fn pending_packets(&self) -> Vec<ReliableDatagramPacket> {
        self.pending_packets_impl()
    }

    pub fn stats(&self) -> ReliableDatagramStats {
        self.stats_impl()
    }
}

impl Default for NetReliableUdpRuntimeManager {
    fn default() -> Self {
        Self::new(ReliableDatagramConfig::default())
    }
}

pub fn net_reliable_udp_runtime_manager() -> NetReliableUdpRuntimeManager {
    NetReliableUdpRuntimeManager::default()
}
