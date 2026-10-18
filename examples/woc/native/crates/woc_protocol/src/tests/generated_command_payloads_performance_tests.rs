use std::{hint::black_box, time::Instant};

use super::*;

const QUERY_COUNT: usize = 100_000;
const SAMPLE_PAIRS: usize = 21;

fn linear_descriptor(id: u16) -> Option<&'static CommandPayloadDescriptor> {
    COMMAND_PAYLOAD_CATALOG.iter().find(|entry| entry.id == id)
}

fn query_id(query: usize) -> u16 {
    if query % 11 == 0 {
        512 + (query % 97) as u16
    } else {
        ((query.wrapping_mul(73) + 41) % 165) as u16
    }
}

fn measure_linear() -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for query in 0..QUERY_COUNT {
        checksum ^= linear_descriptor(black_box(query_id(query)))
            .map(|entry| entry.max_byte_length)
            .unwrap_or(127);
    }
    black_box(checksum);
    started.elapsed().as_nanos() as u64
}

fn measure_binary() -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for query in 0..QUERY_COUNT {
        checksum ^= command_payload_descriptor(black_box(query_id(query)))
            .map(|entry| entry.max_byte_length)
            .unwrap_or(127);
    }
    black_box(checksum);
    started.elapsed().as_nanos() as u64
}

fn sample_csv(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn generated_payload_lookup_matches_linear_oracle_for_every_u16_id() {
    assert_eq!(COMMAND_PAYLOAD_CATALOG.len(), 157);
    assert!(
        COMMAND_PAYLOAD_CATALOG
            .windows(2)
            .all(|pair| pair[0].id < pair[1].id)
    );
    for id in u16::MIN..=u16::MAX {
        assert_eq!(
            command_payload_descriptor(id),
            linear_descriptor(id),
            "{id}"
        );
    }
}

#[test]
#[ignore = "release performance evidence; run through the coordinator"]
fn runtime19_command_payload_binary_lookup_release_benchmark_evidence() {
    let mut linear_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut binary_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            linear_ns.push(measure_linear());
            binary_ns.push(measure_binary());
        } else {
            binary_ns.push(measure_binary());
            linear_ns.push(measure_linear());
        }
    }

    println!(
        "RUNTIME19_COMMAND_PAYLOAD_LOOKUP_PERF descriptors=157 queries_per_sample=100000 \
             sample_pairs=21 sample_order=alternating_linear_first_even \
             percentile_method=nearest_rank threshold_percent=65 linear_ns={} binary_ns={}",
        sample_csv(&linear_ns),
        sample_csv(&binary_ns)
    );
}
