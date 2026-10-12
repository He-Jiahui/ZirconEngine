use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const BATCHES_PER_SAMPLE: usize = 64;
const COMMANDS_PER_BATCH: usize = 4_096;

#[test]
fn optimization_batch_jn_editor653_widget_import_replay_reserves_command_bound() {
    let source = include_str!("../../command_entry.rs");
    let function = source
        .split("fn build_widget_import_replay_commands(")
        .nth(1)
        .expect("widget import replay builder");
    let function = function
        .split("fn widget_import_target_entries(")
        .next()
        .expect("bounded widget import replay function");

    assert!(function.contains("Vec::with_capacity(current.len().saturating_add(target.len()))"));
    assert!(!function.contains("let mut commands = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_jn_editor653_widget_import_replay_capacity_bench() {
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure(false));
            reserved_samples.push(measure(true));
        } else {
            reserved_samples.push(measure(true));
            unreserved_samples.push(measure(false));
        }
    }

    let unreserved_p50_ns = percentile(&unreserved_samples, 50);
    let reserved_p50_ns = percentile(&reserved_samples, 50);
    let unreserved_p95_ns = percentile(&unreserved_samples, 95);
    let reserved_p95_ns = percentile(&reserved_samples, 95);
    println!(
        "EDITOR653_WIDGET_IMPORT_REPLAY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} commands_per_batch={COMMANDS_PER_BATCH} \
unreserved_p50_ns={unreserved_p50_ns} reserved_p50_ns={reserved_p50_ns} \
unreserved_p95_ns={unreserved_p95_ns} reserved_p95_ns={reserved_p95_ns} \
unreserved_raw_ns={} reserved_raw_ns={}",
        sample_csv(&unreserved_samples),
        sample_csv(&reserved_samples),
    );

    assert!(reserved_p95_ns <= unreserved_p95_ns * 80 / 100);
}

#[derive(Clone, Copy)]
struct ReplayCommandFixture([usize; 8]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for batch in 0..BATCHES_PER_SAMPLE {
        let mut commands = if reserve {
            Vec::with_capacity(COMMANDS_PER_BATCH)
        } else {
            Vec::new()
        };
        for command in 0..COMMANDS_PER_BATCH {
            commands.push(ReplayCommandFixture([black_box(batch ^ command); 8]));
        }
        checksum ^= commands[COMMANDS_PER_BATCH - 1].0[0] ^ commands.len();
        black_box(&commands);
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
