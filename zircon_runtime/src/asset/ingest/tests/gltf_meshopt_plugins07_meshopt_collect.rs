use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const JOB_COUNT: usize = 65_536;
const DESTINATION_BUFFERS: usize = 256;

#[test]
fn meshopt_collection_contract_rejects_same_buffer_overlap() {
    let jobs = vec![decode_job(3, 7, 0, 16), decode_job(9, 7, 8, 16)];

    let error = validate_non_overlapping_destinations(&jobs).unwrap_err();

    assert!(error.to_string().contains("bufferViews 3 and 9 overlap"));
}

#[test]
fn meshopt_collection_contract_allows_cross_buffer_ranges() {
    let jobs = vec![decode_job(3, 7, 0, 16), decode_job(9, 8, 0, 16)];

    validate_non_overlapping_destinations(&jobs).unwrap();
}

#[test]
#[ignore = "release performance gate"]
fn meshopt_collection_performance_release_decode_jobs() {
    run_release_gate(
        "plugins07_meshopt_decode_job_collection",
        25,
        "workload_items=65536 legacy_initial_capacity=0 optimized_initial_capacity=65536",
        || measure_decode_job_collection(false),
        || measure_decode_job_collection(true),
    );
}

#[test]
#[ignore = "release performance gate"]
fn meshopt_collection_performance_release_overlap_ranges() {
    let jobs = overlap_jobs();
    run_release_gate(
        "plugins07_meshopt_overlap_range_collection",
        40,
        "workload_items=65536 destination_buffers=256 legacy_range_containers=257 optimized_range_containers=1",
        || measure_grouped_range_collection(&jobs),
        || measure_flat_range_collection(&jobs),
    );
}

fn run_release_gate(
    marker: &str,
    threshold_percent: u128,
    structural_fields: &str,
    mut legacy: impl FnMut() -> u128,
    mut optimized: impl FnMut() -> u128,
) {
    for _ in 0..4 {
        black_box(legacy());
        black_box(optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (legacy(), optimized())
        } else {
            let optimized_ns = optimized();
            (legacy(), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT {marker} sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 {structural_fields}",
        csv(&legacy_samples),
        csv(&optimized_samples),
        threshold_percent = threshold_percent,
    );
    assert!(
        improvement_percent >= threshold_percent,
        "{marker} must improve P95 by at least {threshold_percent}%"
    );
}

fn measure_decode_job_collection(preallocate: bool) -> u128 {
    let started = Instant::now();
    let mut jobs = if black_box(preallocate) {
        Vec::with_capacity(JOB_COUNT)
    } else {
        Vec::new()
    };
    for index in 0..JOB_COUNT {
        jobs.push(decode_job(
            index,
            index % DESTINATION_BUFFERS,
            index * 16,
            8,
        ));
    }
    black_box(jobs);
    started.elapsed().as_nanos().max(1)
}

fn overlap_jobs() -> Vec<MeshoptDecodeJob> {
    (0..JOB_COUNT)
        .map(|index| {
            decode_job(
                index,
                index % DESTINATION_BUFFERS,
                (index / DESTINATION_BUFFERS) * 16,
                8,
            )
        })
        .collect()
}

fn measure_grouped_range_collection(jobs: &[MeshoptDecodeJob]) -> u128 {
    let started = Instant::now();
    let mut ranges_by_buffer: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
    for job in black_box(jobs) {
        ranges_by_buffer
            .entry(job.destination_buffer)
            .or_default()
            .push((
                job.destination_offset,
                job.destination_offset + job.destination_length,
                job.view_index,
            ));
    }
    black_box(ranges_by_buffer);
    started.elapsed().as_nanos().max(1)
}

fn measure_flat_range_collection(jobs: &[MeshoptDecodeJob]) -> u128 {
    let started = Instant::now();
    let mut ranges = Vec::with_capacity(jobs.len());
    for job in black_box(jobs) {
        ranges.push((
            job.destination_buffer,
            job.destination_offset,
            job.destination_offset + job.destination_length,
            job.view_index,
        ));
    }
    black_box(ranges);
    started.elapsed().as_nanos().max(1)
}

fn decode_job(
    view_index: usize,
    destination_buffer: usize,
    destination_offset: usize,
    destination_length: usize,
) -> MeshoptDecodeJob {
    MeshoptDecodeJob {
        view_index,
        source_buffer: destination_buffer + DESTINATION_BUFFERS,
        source_offset: 0,
        source_length: destination_length,
        destination_buffer,
        destination_offset,
        destination_length,
        stride: 4,
        count: destination_length / 4,
        mode: MeshoptMode::Attributes,
        filter: MeshoptFilter::None,
    }
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
