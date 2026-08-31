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
mod exact_delivery_capacity_tests {
    use std::{hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::{
        ReliableDatagramPacket, ReliableDatagramSimulationProfile,
    };

    use super::{NetReliableUdpRuntimeManager, simulated_delivery_capacities};

    const BENCHMARK_PACKET_COUNT: usize = 8_192;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;
    const DROP_INTERVAL: u64 = 3;

    #[test]
    fn exact_delivery_capacity_respects_counter_phase_and_drop_cadence() {
        let manager = NetReliableUdpRuntimeManager::default();
        manager.set_simulation_profile(
            ReliableDatagramSimulationProfile::new().with_drop_every_nth_packet(DROP_INTERVAL),
        );
        let warmup = manager.simulate_outbound_delivery([packet(0)]);
        assert_eq!(warmup.delivered_packets.len(), 1);

        assert_eq!(
            simulated_delivery_capacities(1, Some(DROP_INTERVAL), 5),
            Some((3, 2))
        );
        let delivery = manager.simulate_outbound_delivery((1..=5).map(packet));
        assert_eq!(sequences(&delivery.delivered_packets), vec![1, 3, 4]);
        assert_eq!(sequences(&delivery.dropped_packets), vec![2, 5]);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn exact_delivery_capacity_release_benchmark_evidence() {
        let seed = (0..BENCHMARK_PACKET_COUNT as u64)
            .map(packet)
            .collect::<Vec<_>>();
        let legacy_equivalence = legacy_partition(seed.clone()).0;
        let optimized_equivalence = optimized_partition(seed.clone()).0;
        assert_eq!(legacy_equivalence, optimized_equivalence);

        let legacy_capacity_growths = legacy_partition(seed.clone()).1;
        let optimized_capacity_growths = optimized_partition(seed.clone()).1;
        assert!(legacy_capacity_growths > 2);
        assert_eq!(optimized_capacity_growths, 0);

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&seed));
                optimized_samples.push(measure_optimized(&seed));
            } else {
                optimized_samples.push(measure_optimized(&seed));
                legacy_samples.push(measure_legacy(&seed));
            }
        }

        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT task=plugins10_exact_rudp_delivery_capacity packets={BENCHMARK_PACKET_COUNT} drop_interval={DROP_INTERVAL} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_capacity_growths_per_sample={legacy_capacity_growths} optimized_capacity_growths_per_sample={optimized_capacity_growths} threshold_percent=15 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_samples),
            raw_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
            "exact capacity P95 {optimized_p95}ns did not improve legacy {legacy_p95}ns by 15%"
        );
    }

    fn packet(sequence: u64) -> ReliableDatagramPacket {
        ReliableDatagramPacket::new(sequence, "benchmark-channel", vec![sequence as u8; 64])
    }

    fn sequences(packets: &[ReliableDatagramPacket]) -> Vec<u64> {
        packets.iter().map(|packet| packet.sequence).collect()
    }

    fn legacy_partition(
        packets: Vec<ReliableDatagramPacket>,
    ) -> (
        (Vec<ReliableDatagramPacket>, Vec<ReliableDatagramPacket>),
        usize,
    ) {
        partition_with_capacities(packets, 0, 0)
    }

    fn optimized_partition(
        packets: Vec<ReliableDatagramPacket>,
    ) -> (
        (Vec<ReliableDatagramPacket>, Vec<ReliableDatagramPacket>),
        usize,
    ) {
        let (delivered_capacity, dropped_capacity) =
            simulated_delivery_capacities(0, Some(DROP_INTERVAL), packets.len())
                .expect("benchmark packet count should fit the simulation counter");
        partition_with_capacities(packets, delivered_capacity, dropped_capacity)
    }

    fn partition_with_capacities(
        packets: Vec<ReliableDatagramPacket>,
        delivered_capacity: usize,
        dropped_capacity: usize,
    ) -> (
        (Vec<ReliableDatagramPacket>, Vec<ReliableDatagramPacket>),
        usize,
    ) {
        let mut delivered = Vec::with_capacity(delivered_capacity);
        let mut dropped = Vec::with_capacity(dropped_capacity);
        let mut counter = 0u64;
        let mut capacity_growths = 0;
        for packet in packets {
            counter += 1;
            let destination = if counter % DROP_INTERVAL == 0 {
                &mut dropped
            } else {
                &mut delivered
            };
            let previous_capacity = destination.capacity();
            destination.push(packet);
            capacity_growths += usize::from(destination.capacity() != previous_capacity);
        }
        ((delivered, dropped), capacity_growths)
    }

    fn measure_legacy(seed: &[ReliableDatagramPacket]) -> u128 {
        let packets = seed.to_vec();
        let start = Instant::now();
        let result = legacy_partition(black_box(packets));
        let elapsed = start.elapsed().as_nanos();
        black_box(result);
        elapsed
    }

    fn measure_optimized(seed: &[ReliableDatagramPacket]) -> u128 {
        let packets = seed.to_vec();
        let start = Instant::now();
        let result = optimized_partition(black_box(packets));
        let elapsed = start.elapsed().as_nanos();
        black_box(result);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u128]) -> String {
        format!(
            "[{}]",
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
