use std::{hint::black_box, time::Instant};

use woc_protocol::{EntityRef, MovementFrameBatch, MovementInputFlags};

use super::*;

const FRAMES_PER_BATCH: usize = 32_768;
const ITERATIONS: usize = 4;
const SAMPLE_PAIRS: usize = 21;
const THRESHOLD_PERCENT: u64 = 35;

fn fixture() -> Vec<MovementFrame> {
    (0..FRAMES_PER_BATCH)
        .rev()
        .map(|index| MovementFrame {
            actor: EntityRef {
                id: index as u64 + 1,
                generation: (index % 7) as u32,
            },
            sequence: index as u32 + 1,
            flags: MovementInputFlags {
                forward: true,
                turn_right: true,
                jump: index % 2 == 0,
                ..MovementInputFlags::default()
            },
            facing: Some(index as f64 / FRAMES_PER_BATCH as f64),
        })
        .collect()
}

fn prepared_inputs(fixture: &[MovementFrame]) -> Vec<Vec<MovementFrame>> {
    (0..ITERATIONS).map(|_| fixture.to_vec()).collect()
}

fn consume(frames: &[MovementFrame]) -> u64 {
    frames
        .iter()
        .fold(0x517c_c1b7_2722_0a95, |checksum, frame| {
            checksum.rotate_left(7)
                ^ frame.actor.id
                ^ u64::from(frame.actor.generation).rotate_left(11)
                ^ u64::from(frame.sequence).rotate_left(23)
                ^ u64::from(frame.flags.forward)
        })
}

fn measure_legacy(fixture: &[MovementFrame]) -> u64 {
    let inputs = prepared_inputs(fixture);
    let started = Instant::now();
    let mut checksum = 0;
    for movement_frames in inputs {
        let movement_batch = MovementFrameBatch::new(movement_frames.clone())
            .expect("fixture must produce a valid movement batch");
        let diagnostic = movement_batch.frames().to_vec();
        let runtime = diagnostic.clone();
        checksum = checksum
            .wrapping_add(consume(&diagnostic))
            .wrapping_add(consume(&runtime).rotate_left(17));
    }
    black_box(checksum);
    started.elapsed().as_nanos() as u64
}

fn measure_transferred(fixture: &[MovementFrame]) -> u64 {
    let inputs = prepared_inputs(fixture);
    let started = Instant::now();
    let mut checksum = 0;
    for mut movement_frames in inputs {
        canonicalize_pending_movement(&mut movement_frames)
            .expect("fixture must fit the protocol bound");
        let diagnostic = movement_frames.clone();
        let runtime = movement_frames;
        checksum = checksum
            .wrapping_add(consume(&diagnostic))
            .wrapping_add(consume(&runtime).rotate_left(17));
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

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn reduction_percent(legacy: u64, transferred: u64) -> u64 {
    legacy.saturating_sub(transferred).saturating_mul(100) / legacy.max(1)
}

#[test]
#[ignore = "release performance evidence; run through the coordinator"]
fn woc_app05_movement_transfer_release_benchmark_evidence() {
    let fixture = fixture();
    let legacy = MovementFrameBatch::new(fixture.clone())
        .expect("fixture must produce a valid movement batch");
    let mut transferred = fixture.clone();
    canonicalize_pending_movement(&mut transferred)
        .expect("fixture must fit the protocol bound");
    assert_eq!(legacy.frames(), transferred.as_slice());

    for _ in 0..4 {
        black_box(measure_legacy(&fixture));
        black_box(measure_transferred(&fixture));
    }

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut transferred_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&fixture));
            transferred_ns.push(measure_transferred(&fixture));
        } else {
            transferred_ns.push(measure_transferred(&fixture));
            legacy_ns.push(measure_legacy(&fixture));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let transferred_p50_ns = nearest_rank(&transferred_ns, 50);
    let transferred_p95_ns = nearest_rank(&transferred_ns, 95);
    let p50_reduction_percent = reduction_percent(legacy_p50_ns, transferred_p50_ns);
    let p95_reduction_percent = reduction_percent(legacy_p95_ns, transferred_p95_ns);

    println!(
        "WOC_APP05_MOVEMENT_TRANSFER_PERF frames_per_batch=32768 iterations=4 \
             sample_pairs=21 sample_order=alternating_legacy_first_even \
             percentile_method=nearest_rank threshold_percent=35 \
             legacy_full_vector_copies=3 transferred_full_vector_copies=1 \
             copy_reduction_percent=66 legacy_p50_ns={legacy_p50_ns} \
             transferred_p50_ns={transferred_p50_ns} \
             p50_reduction_percent={p50_reduction_percent} \
             legacy_p95_ns={legacy_p95_ns} transferred_p95_ns={transferred_p95_ns} \
             p95_reduction_percent={p95_reduction_percent} \
             legacy_ns={} transferred_ns={}",
        sample_csv(&legacy_ns),
        sample_csv(&transferred_ns)
    );

    assert!(
        p50_reduction_percent >= THRESHOLD_PERCENT,
        "ownership transfer must improve P50 by at least {THRESHOLD_PERCENT}%: \
             legacy={legacy_p50_ns}ns transferred={transferred_p50_ns}ns"
    );
    assert!(
        p95_reduction_percent >= THRESHOLD_PERCENT,
        "ownership transfer must improve P95 by at least {THRESHOLD_PERCENT}%: \
             legacy={legacy_p95_ns}ns transferred={transferred_p95_ns}ns"
    );
}
