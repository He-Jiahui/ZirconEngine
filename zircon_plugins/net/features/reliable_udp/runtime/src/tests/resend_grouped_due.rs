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
