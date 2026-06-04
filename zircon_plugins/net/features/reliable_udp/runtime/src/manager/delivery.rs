use zircon_runtime::core::framework::net::{
    ReliableDatagramDeliveryReport, ReliableDatagramPacket, ReliableDatagramRecoveryState,
    ReliableDatagramSimulationProfile,
};

use super::NetReliableUdpRuntimeManager;

impl NetReliableUdpRuntimeManager {
    pub(in crate::manager) fn set_simulation_profile_impl(
        &self,
        profile: ReliableDatagramSimulationProfile,
    ) {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        state.simulation_profile = profile;
        state.simulated_packet_counter = 0;
    }

    pub(in crate::manager) fn simulate_outbound_delivery_impl(
        &self,
        packets: impl IntoIterator<Item = ReliableDatagramPacket>,
    ) -> ReliableDatagramDeliveryReport {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let mut delivered = Vec::new();
        let mut dropped = Vec::new();
        for packet in packets {
            state.simulated_packet_counter += 1;
            if state.should_drop_simulated_packet() {
                state.stats.dropped_packets += 1;
                state.dropped_packets_since_recovery += 1;
                dropped.push(packet);
            } else {
                delivered.push(packet);
            }
        }
        reorder_delivered_packets(&mut delivered, state.simulation_profile.reorder_window);
        state.update_recovery_after_delivery();
        ReliableDatagramDeliveryReport::new(delivered, dropped, state.recovery_report())
    }
}

impl super::state::NetReliableUdpRuntimeState {
    pub(in crate::manager) fn should_drop_simulated_packet(&self) -> bool {
        self.simulation_profile
            .drop_every_nth_packet
            .is_some_and(|packet_interval| self.simulated_packet_counter % packet_interval == 0)
    }

    pub(in crate::manager) fn update_recovery_after_delivery(&mut self) {
        if self.recovery_state == ReliableDatagramRecoveryState::Disconnected {
            return;
        }
        self.recovery_state = match self.simulation_profile.recovery_drop_threshold {
            Some(threshold) if self.dropped_packets_since_recovery >= threshold => {
                ReliableDatagramRecoveryState::Recovering
            }
            _ => ReliableDatagramRecoveryState::Connected,
        };
        self.recovery_diagnostic = (self.recovery_state
            == ReliableDatagramRecoveryState::Recovering)
            .then(|| "drop threshold reached".to_string());
    }
}

fn reorder_delivered_packets(packets: &mut Vec<ReliableDatagramPacket>, reorder_window: usize) {
    if reorder_window <= 1 {
        return;
    }
    for chunk in packets.chunks_mut(reorder_window) {
        chunk.reverse();
    }
}
