use std::collections::{HashMap, HashSet, VecDeque};

use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramPacket, ReliableDatagramRecoveryState,
};

use super::{NetReliableUdpRuntimeManager, RESEND_ATTEMPT_CAP_DIAGNOSTIC};
use crate::ReliableUdpWireHeader;

impl NetReliableUdpRuntimeManager {
    pub(in crate::manager) fn acknowledge_impl(&self, ack: ReliableDatagramAck) -> usize {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let before = state.outbound.len();
        state
            .outbound
            .retain(|packet| packet.sequence != ack.sequence);
        state.resend_state.remove(&ack.sequence);
        let removed = before - state.outbound.len();
        state.stats.received_packets += removed as u64;
        removed
    }

    pub(in crate::manager) fn acknowledge_wire_header_impl(
        &self,
        header: ReliableUdpWireHeader,
    ) -> usize {
        let acked_sequences = header.acked_sequences().into_iter().collect::<HashSet<_>>();
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let before = state.outbound.len();
        let acknowledged = state
            .outbound
            .iter()
            .filter(|packet| acked_sequences.contains(&(packet.sequence as u16)))
            .map(|packet| packet.sequence)
            .collect::<HashSet<_>>();
        state
            .outbound
            .retain(|packet| !acknowledged.contains(&packet.sequence));
        for sequence in &acknowledged {
            state.resend_state.remove(sequence);
        }
        let removed = before - state.outbound.len();
        state.stats.received_packets += removed as u64;
        removed
    }

    pub(in crate::manager) fn resend_pending_impl(
        &self,
        max_packets: usize,
    ) -> Vec<ReliableDatagramPacket> {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let packets = state
            .outbound
            .iter()
            .take(max_packets)
            .cloned()
            .collect::<Vec<_>>();
        state.stats.resent_packets += packets.len() as u64;
        packets
    }

    pub(in crate::manager) fn resend_due_impl(&self, now_ms: u64) -> Vec<ReliableDatagramPacket> {
        self.resend_due_with_byte_budget_impl(now_ms, usize::MAX)
    }

    pub(in crate::manager) fn resend_due_with_byte_budget_impl(
        &self,
        now_ms: u64,
        max_payload_bytes: usize,
    ) -> Vec<ReliableDatagramPacket> {
        let mut state = self
            .state
            .lock()
            .expect("net reliable UDP state mutex poisoned");
        let mut due_sequences = state.due_resend_sequences(now_ms);
        due_sequences.sort_unstable();
        if due_sequences.is_empty() {
            return Vec::new();
        }

        let max_attempts = state.config.max_resend_attempts;
        let (capped_sequences, resend_sequences): (Vec<_>, Vec<_>) =
            due_sequences.into_iter().partition(|sequence| {
                state
                    .resend_state
                    .get(sequence)
                    .is_some_and(|resend_state| resend_state.attempts >= max_attempts)
            });
        let mut packets_by_sequence = due_packets_by_sequence(&state.outbound, &resend_sequences);
        let mut due_packets = Vec::new();
        let mut remaining_bytes = max_payload_bytes;
        for sequence in resend_sequences {
            let packets = packets_by_sequence.remove(&sequence).unwrap_or_default();
            let byte_cost = packets
                .iter()
                .map(|packet| packet.payload.len())
                .sum::<usize>();
            if byte_cost > remaining_bytes {
                continue;
            }

            let resend_state = state.resend_state.entry(sequence).or_default();
            resend_state.attempts += 1;
            resend_state.last_sent_at_ms = now_ms;
            remaining_bytes -= byte_cost;
            due_packets.extend(packets);
        }

        if !capped_sequences.is_empty() {
            state.drop_capped_sequences(&capped_sequences);
            state.recovery_state = ReliableDatagramRecoveryState::Disconnected;
            state.recovery_diagnostic = Some(RESEND_ATTEMPT_CAP_DIAGNOSTIC.to_string());
        }
        state.stats.resent_packets += due_packets.len() as u64;
        due_packets
    }
}

fn due_packets_by_sequence(
    outbound: &VecDeque<ReliableDatagramPacket>,
    due_sequences: &[u64],
) -> HashMap<u64, Vec<ReliableDatagramPacket>> {
    let due_sequences = due_sequences.iter().copied().collect::<HashSet<_>>();
    let mut packets_by_sequence = HashMap::with_capacity(due_sequences.len());
    for packet in outbound {
        if due_sequences.contains(&packet.sequence) {
            packets_by_sequence
                .entry(packet.sequence)
                .or_insert_with(Vec::new)
                .push(packet.clone());
        }
    }
    packets_by_sequence
}

impl super::state::NetReliableUdpRuntimeState {
    fn due_resend_sequences(&self, now_ms: u64) -> Vec<u64> {
        let resend_timeout_ms = self.config.resend_timeout_ms;
        if resend_timeout_ms == 0 {
            return self.resend_state.keys().copied().collect();
        }
        self.resend_state
            .iter()
            .filter_map(|(sequence, resend_state)| {
                now_ms
                    .saturating_sub(resend_state.last_sent_at_ms)
                    .ge(&resend_timeout_ms)
                    .then_some(*sequence)
            })
            .collect()
    }

    fn drop_capped_sequences(&mut self, sequences: &[u64]) {
        self.outbound
            .retain(|packet| !sequences.contains(&packet.sequence));
        for sequence in sequences {
            self.resend_state.remove(sequence);
        }
    }
}

#[cfg(test)]
mod grouped_due_packet_tests {
    use std::{collections::VecDeque, hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::ReliableDatagramPacket;

    use super::due_packets_by_sequence;

    const BENCHMARK_PACKET_COUNT: usize = 1_024;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn grouped_due_packets_match_sequence_ordered_legacy_scan() {
        let outbound = VecDeque::from([packet(2, 0), packet(1, 0), packet(2, 1), packet(3, 0)]);
        let due_sequences = [1, 2];

        assert_eq!(
            grouped_due_packets(&outbound, &due_sequences),
            legacy_due_packets(&outbound, &due_sequences)
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn grouped_due_packets_release_benchmark_evidence() {
        let outbound = (1..=BENCHMARK_PACKET_COUNT as u64)
            .rev()
            .map(|sequence| packet(sequence, 0))
            .collect::<VecDeque<_>>();
        let due_sequences = (1..=BENCHMARK_PACKET_COUNT as u64).collect::<Vec<_>>();
        assert_eq!(
            grouped_due_packets(&outbound, &due_sequences),
            legacy_due_packets(&outbound, &due_sequences)
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_checksum(&outbound, &due_sequences),
            || grouped_checksum(&outbound, &due_sequences),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_raw_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_raw_ns = benchmark_samples_csv(&optimized_samples);
        let legacy_packet_inspections = BENCHMARK_PACKET_COUNT * BENCHMARK_PACKET_COUNT;

        println!(
            "PERF_RESULT task=plugins10_grouped_due_packets packets={BENCHMARK_PACKET_COUNT} due_sequences={BENCHMARK_PACKET_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_packet_inspections_per_sample={legacy_packet_inspections} optimized_packet_inspections_per_sample={BENCHMARK_PACKET_COUNT} threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_raw_ns} optimized_raw_ns={optimized_raw_ns}"
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    fn packet(sequence: u64, fragment_index: u16) -> ReliableDatagramPacket {
        ReliableDatagramPacket::new(sequence, "state", vec![sequence as u8; 32])
            .with_fragment(fragment_index, 2)
    }

    fn legacy_due_packets(
        outbound: &VecDeque<ReliableDatagramPacket>,
        due_sequences: &[u64],
    ) -> Vec<ReliableDatagramPacket> {
        due_sequences
            .iter()
            .flat_map(|sequence| {
                outbound
                    .iter()
                    .filter(move |packet| packet.sequence == *sequence)
                    .cloned()
            })
            .collect()
    }

    fn grouped_due_packets(
        outbound: &VecDeque<ReliableDatagramPacket>,
        due_sequences: &[u64],
    ) -> Vec<ReliableDatagramPacket> {
        let mut packets_by_sequence = due_packets_by_sequence(outbound, due_sequences);
        due_sequences
            .iter()
            .flat_map(|sequence| packets_by_sequence.remove(sequence).unwrap_or_default())
            .collect()
    }

    fn legacy_checksum(
        outbound: &VecDeque<ReliableDatagramPacket>,
        due_sequences: &[u64],
    ) -> usize {
        black_box(
            legacy_due_packets(black_box(outbound), black_box(due_sequences))
                .iter()
                .map(|packet| packet.payload.len())
                .sum(),
        )
    }

    fn grouped_checksum(
        outbound: &VecDeque<ReliableDatagramPacket>,
        due_sequences: &[u64],
    ) -> usize {
        black_box(
            grouped_due_packets(black_box(outbound), black_box(due_sequences))
                .iter()
                .map(|packet| packet.payload.len())
                .sum(),
        )
    }

    fn benchmark_paired_samples(
        mut legacy: impl FnMut() -> usize,
        mut optimized: impl FnMut() -> usize,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
        let started = Instant::now();
        let checksum = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        assert_eq!(checksum, BENCHMARK_PACKET_COUNT * 32);
        elapsed
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }
}
