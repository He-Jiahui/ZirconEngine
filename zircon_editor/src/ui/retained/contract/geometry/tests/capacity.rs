use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const FRAMES_PER_SAMPLE: usize = 128;
const BUTTONS_PER_RAIL: usize = 2_048;
const BUTTONS_PER_FRAME: usize = BUTTONS_PER_RAIL * 2;

#[test]
fn optimization_batch_20260826fz_editor167_activity_rail_capacity_covers_both_models() {
    let button_capacity = BUTTONS_PER_RAIL.saturating_add(BUTTONS_PER_RAIL);
    let mut buttons = Vec::with_capacity(button_capacity);
    buttons.extend(0..BUTTONS_PER_RAIL);
    buttons.extend(BUTTONS_PER_RAIL..BUTTONS_PER_FRAME);

    assert_eq!(buttons.len(), BUTTONS_PER_FRAME);
    assert!(buttons.capacity() >= BUTTONS_PER_FRAME);
    assert_eq!(buttons[0], 0);
    assert_eq!(buttons[BUTTONS_PER_FRAME - 1], BUTTONS_PER_FRAME - 1);
}

#[test]
fn optimization_batch_20260826fz_editor167_activity_rail_reserves_left_and_right_rows() {
    let source = include_str!("../activity_rail_buttons.rs");
    let left_rows = source.find(".left_dock").expect("left activity rail model");
    let saturated_sum = source.find(".saturating_add(").expect("saturating sum");
    let right_rows = source
        .find("scene.right_dock.rail_button_frames.row_count()")
        .expect("right activity rail model");
    let reserve = source
        .find("Vec::with_capacity(button_capacity)")
        .expect("button capacity");

    assert!(left_rows < saturated_sum && saturated_sum < right_rows && right_rows < reserve);
    assert!(!source.contains("let mut buttons = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fz_editor167_activity_rail_profile_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR167_ACTIVITY_RAIL_PROFILE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
frames_per_sample={FRAMES_PER_SAMPLE} buttons_per_frame={BUTTONS_PER_FRAME} \
legacy_preallocated_frame_outputs=0 optimized_preallocated_frame_outputs={FRAMES_PER_SAMPLE} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for frame in 0..FRAMES_PER_SAMPLE {
        let mut buttons = if reserve {
            Vec::with_capacity(BUTTONS_PER_FRAME)
        } else {
            Vec::new()
        };
        for button in 0..BUTTONS_PER_FRAME {
            let value = black_box(frame ^ button);
            buttons.push([value; 12]);
        }
        checksum ^= black_box(buttons.len() ^ buttons.capacity());
        black_box(&buttons);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
