use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 4_194_304;
const MACHINE_KEYS: usize = 4_096;

#[test]
fn optimization_batch_jn_runtime653_machine_insert_reuses_entry_lookup() {
    let source = include_str!("../../registry.rs");
    let function = source
        .split("fn machine_or_insert_mut<T: StateSpec>(")
        .nth(1)
        .expect("machine insertion helper");

    assert!(function.contains(".downcast_mut::<StateMachine<T>>()"));
    assert!(!function.contains("self.machine_mut::<T>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_jn_runtime653_single_machine_lookup_bench() {
    for _ in 0..4 {
        black_box(measure(false));
        black_box(measure(true));
    }

    let mut double_lookup_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut single_lookup_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            double_lookup_samples.push(measure(false));
            single_lookup_samples.push(measure(true));
        } else {
            single_lookup_samples.push(measure(true));
            double_lookup_samples.push(measure(false));
        }
    }

    let double_lookup_p50_ns = percentile(&double_lookup_samples, 50);
    let single_lookup_p50_ns = percentile(&single_lookup_samples, 50);
    let double_lookup_p95_ns = percentile(&double_lookup_samples, 95);
    let single_lookup_p95_ns = percentile(&single_lookup_samples, 95);
    println!(
        "RUNTIME653_SINGLE_MACHINE_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} machine_keys={MACHINE_KEYS} \
double_lookup_p50_ns={double_lookup_p50_ns} single_lookup_p50_ns={single_lookup_p50_ns} \
double_lookup_p95_ns={double_lookup_p95_ns} single_lookup_p95_ns={single_lookup_p95_ns} \
double_lookup_raw_ns={} single_lookup_raw_ns={}",
        sample_csv(&double_lookup_samples),
        sample_csv(&single_lookup_samples),
    );

    assert!(single_lookup_p95_ns <= double_lookup_p95_ns * 75 / 100);
}

fn measure(single_lookup: bool) -> u128 {
    let mut machines = (0..MACHINE_KEYS)
        .map(|key| (key as u64, key as u64))
        .collect::<HashMap<_, _>>();
    let started = Instant::now();
    let mut checksum = 0u64;
    for lookup in 0..LOOKUPS_PER_SAMPLE {
        let key = black_box((lookup % MACHINE_KEYS) as u64);
        let machine = if single_lookup {
            machines.entry(key).or_default()
        } else {
            machines.entry(key).or_default();
            machines.get_mut(&key).expect("occupied machine key")
        };
        *machine = machine.wrapping_add(1);
        checksum ^= *machine;
    }
    black_box(checksum ^ machines.len() as u64);
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
