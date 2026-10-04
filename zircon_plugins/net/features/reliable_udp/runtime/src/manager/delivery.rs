//! 提供确定性的丢包/重排模拟，用于验证重发、顺序投递和恢复状态推演。
//! 这是测试/模拟边界，不执行真实 UDP I/O；修改 profile 会重置模拟计数。

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
        let packets = packets.into_iter();
        let (lower_bound, upper_bound) = packets.size_hint();
        let exact_packet_count = upper_bound.filter(|upper_bound| *upper_bound == lower_bound);
        let (delivered_capacity, dropped_capacity) = exact_packet_count
            .and_then(|packet_count| {
                simulated_delivery_capacities(
                    state.simulated_packet_counter,
                    state.simulation_profile.drop_every_nth_packet,
                    packet_count,
                )
            })
            .unwrap_or_default();
        let mut delivered = Vec::with_capacity(delivered_capacity);
        let mut dropped = Vec::with_capacity(dropped_capacity);
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
    // BUG: [CR-PLUGIN-NET-0022] profile 可由公开字段/serde 携带 Some(0)，这里取模会 panic；setter 的清洗未覆盖该入口。
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

fn simulated_delivery_capacities(
    simulated_packet_counter: u64,
    drop_every_nth_packet: Option<u64>,
    packet_count: usize,
) -> Option<(usize, usize)> {
    let Some(drop_interval) = drop_every_nth_packet else {
        return Some((packet_count, 0));
    };
    if drop_interval == 0 {
        return None;
    }

    let final_packet_counter =
        simulated_packet_counter.checked_add(packet_count.try_into().ok()?)?;
    let dropped_count = (final_packet_counter / drop_interval)
        .checked_sub(simulated_packet_counter / drop_interval)?
        .try_into()
        .ok()?;
    Some((packet_count.checked_sub(dropped_count)?, dropped_count))
}

fn reorder_delivered_packets(packets: &mut Vec<ReliableDatagramPacket>, reorder_window: usize) {
    if reorder_window <= 1 {
        return;
    }
    for chunk in packets.chunks_mut(reorder_window) {
        chunk.reverse();
    }
}

#[cfg(test)]
#[path = "tests/delivery_exact_delivery_capacity_tests.rs"]
mod exact_delivery_capacity_tests;
