use zircon_runtime::core::framework::net::{ReliableDatagramPacket, ReliableDatagramReceiveStatus};

// Keeps partial datagrams in fragment-index order so out-of-order delivery can be
// reassembled without leaking runtime-owned buffers through the public contract.
#[derive(Debug)]
pub(in crate::manager) struct InboundFragmentAssembly {
    channel: String,
    fragment_count: u16,
    fragments: Vec<Option<Vec<u8>>>,
}

impl InboundFragmentAssembly {
    pub(in crate::manager) fn new(packet: &ReliableDatagramPacket) -> Self {
        Self {
            channel: packet.channel.clone(),
            fragment_count: packet.fragment_count,
            fragments: vec![None; packet.fragment_count as usize],
        }
    }

    pub(in crate::manager) fn insert(
        &mut self,
        packet: ReliableDatagramPacket,
    ) -> ReliableDatagramReceiveStatus {
        if packet.fragment_count != self.fragment_count
            || packet.channel != self.channel
            || packet.fragment_index >= self.fragment_count
        {
            return ReliableDatagramReceiveStatus::InvalidFragment;
        }
        let fragment = &mut self.fragments[packet.fragment_index as usize];
        if fragment.is_some() {
            return ReliableDatagramReceiveStatus::DuplicateFragment;
        }
        *fragment = Some(packet.payload);
        if self.fragments.iter().all(Option::is_some) {
            ReliableDatagramReceiveStatus::Reassembled
        } else {
            ReliableDatagramReceiveStatus::AcceptedFragment
        }
    }

    pub(in crate::manager) fn payload(&self) -> Vec<u8> {
        self.fragments
            .iter()
            .flat_map(|fragment| fragment.as_deref().unwrap_or_default())
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod moved_fragment_payload_tests {
    use std::{hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::{
        ReliableDatagramPacket, ReliableDatagramReceiveStatus,
    };

    use super::InboundFragmentAssembly;

    const BENCHMARK_FRAGMENT_COUNT: usize = 128;
    const BENCHMARK_FRAGMENT_BYTES: usize = 65_536;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn moved_fragment_payload_preserves_reassembly_order() {
        let first = ReliableDatagramPacket::new(7, "state", b"abc".to_vec()).with_fragment(0, 2);
        let second = ReliableDatagramPacket::new(7, "state", b"def".to_vec()).with_fragment(1, 2);
        let mut assembly = InboundFragmentAssembly::new(&first);

        assert_eq!(
            assembly.insert(first),
            ReliableDatagramReceiveStatus::AcceptedFragment
        );
        assert_eq!(
            assembly.insert(second),
            ReliableDatagramReceiveStatus::Reassembled
        );
        assert_eq!(assembly.payload(), b"abcdef");
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn moved_fragment_payload_release_benchmark_evidence() {
        let packets = benchmark_packets();
        assert_eq!(
            legacy_insert_batch(packets.clone()),
            moved_insert_batch(packets.clone())
        );
        let (legacy_samples, optimized_samples) = benchmark_paired_samples(&packets);
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_raw_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_raw_ns = benchmark_samples_csv(&optimized_samples);
        let payload_bytes = BENCHMARK_FRAGMENT_COUNT * BENCHMARK_FRAGMENT_BYTES;

        println!(
            "PERF_RESULT task=plugins10_moved_fragment_payload fragments={BENCHMARK_FRAGMENT_COUNT} fragment_bytes={BENCHMARK_FRAGMENT_BYTES} payload_bytes_per_sample={payload_bytes} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_payload_clone_bytes_per_sample={payload_bytes} optimized_payload_clone_bytes_per_sample=0 legacy_payload_clones_per_sample={BENCHMARK_FRAGMENT_COUNT} optimized_payload_clones_per_sample=0 threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_raw_ns} optimized_raw_ns={optimized_raw_ns}"
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_packets() -> Vec<ReliableDatagramPacket> {
        (0..BENCHMARK_FRAGMENT_COUNT)
            .map(|index| {
                ReliableDatagramPacket::new(
                    11,
                    "benchmark-channel",
                    vec![index as u8; BENCHMARK_FRAGMENT_BYTES],
                )
                .with_fragment(index as u16, BENCHMARK_FRAGMENT_COUNT as u16)
            })
            .collect()
    }

    fn legacy_insert_batch(packets: Vec<ReliableDatagramPacket>) -> usize {
        let mut assembly = InboundFragmentAssembly::new(&packets[0]);
        for packet in &packets {
            black_box(legacy_insert(&mut assembly, black_box(packet)));
        }
        black_box(stored_payload_bytes(&assembly))
    }

    fn moved_insert_batch(packets: Vec<ReliableDatagramPacket>) -> usize {
        let mut assembly = InboundFragmentAssembly::new(&packets[0]);
        for packet in packets {
            black_box(assembly.insert(black_box(packet)));
        }
        black_box(stored_payload_bytes(&assembly))
    }

    fn legacy_insert(
        assembly: &mut InboundFragmentAssembly,
        packet: &ReliableDatagramPacket,
    ) -> ReliableDatagramReceiveStatus {
        if packet.fragment_count != assembly.fragment_count
            || packet.channel != assembly.channel
            || packet.fragment_index >= assembly.fragment_count
        {
            return ReliableDatagramReceiveStatus::InvalidFragment;
        }
        let fragment = &mut assembly.fragments[packet.fragment_index as usize];
        if fragment.is_some() {
            return ReliableDatagramReceiveStatus::DuplicateFragment;
        }
        *fragment = Some(packet.payload.clone());
        if assembly.fragments.iter().all(Option::is_some) {
            ReliableDatagramReceiveStatus::Reassembled
        } else {
            ReliableDatagramReceiveStatus::AcceptedFragment
        }
    }

    fn stored_payload_bytes(assembly: &InboundFragmentAssembly) -> usize {
        assembly
            .fragments
            .iter()
            .filter_map(Option::as_ref)
            .map(Vec::len)
            .sum()
    }

    fn benchmark_paired_samples(packets: &[ReliableDatagramPacket]) -> (Vec<u128>, Vec<u128>) {
        black_box(benchmark_sample(packets, legacy_insert_batch));
        black_box(benchmark_sample(packets, moved_insert_batch));
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(packets, legacy_insert_batch));
                optimized_samples.push(benchmark_sample(packets, moved_insert_batch));
            } else {
                optimized_samples.push(benchmark_sample(packets, moved_insert_batch));
                legacy_samples.push(benchmark_sample(packets, legacy_insert_batch));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(
        packets: &[ReliableDatagramPacket],
        insert: fn(Vec<ReliableDatagramPacket>) -> usize,
    ) -> u128 {
        let packets = packets.to_vec();
        let started = Instant::now();
        let stored_bytes = black_box(insert(packets));
        let elapsed = started.elapsed().as_nanos();
        assert_eq!(
            stored_bytes,
            BENCHMARK_FRAGMENT_COUNT * BENCHMARK_FRAGMENT_BYTES
        );
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
