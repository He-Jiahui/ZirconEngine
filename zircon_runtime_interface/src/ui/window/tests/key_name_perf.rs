use super::*;

fn physical_key_name_formatting(key_code: u32) -> String {
    if let Some(name) = named_keyboard_key(key_code) {
        name.to_string()
    } else {
        format!("KeyCode{key_code}")
    }
}

fn logical_key_name_formatting(key_code: u32) -> String {
    named_keyboard_key(key_code)
        .map(str::to_string)
        .unwrap_or_else(|| key_code.to_string())
}

#[test]
fn runtime_interface03_batch55_61_physical_key_name_preserves_formatted_output() {
    for key_code in [0, 1, 9, 10, 13, 32, 48, 65, 90, 91, 999, 65_535, u32::MAX] {
        assert_eq!(
            physical_key_name(key_code),
            physical_key_name_formatting(key_code),
        );
    }
}

#[test]
#[ignore = "release-only stack physical key-name benchmark"]
fn runtime_interface03_batch55_61_stack_physical_key_name_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BUILD_COUNT: usize = 1_000_000;
    const SAMPLE_COUNT: usize = 11;
    const UNKNOWN_KEY_CODE: u32 = u32::MAX;
    let mut formatting_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut stack_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_formatting = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(physical_key_name_formatting(black_box(UNKNOWN_KEY_CODE)));
            }
            started.elapsed().as_nanos()
        };
        let measure_stack = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(physical_key_name(black_box(UNKNOWN_KEY_CODE)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            formatting_samples.push(measure_formatting());
            stack_samples.push(measure_stack());
        } else {
            stack_samples.push(measure_stack());
            formatting_samples.push(measure_formatting());
        }
    }

    formatting_samples.sort_unstable();
    stack_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STACK_PHYSICAL_KEY_NAME_BENCH_V1 builds={BUILD_COUNT} digits=10 samples={SAMPLE_COUNT} formatting_p95_ns={} stack_p95_ns={}",
        formatting_samples[p95], stack_samples[p95],
    );
    assert!(
        stack_samples[p95].saturating_mul(5) <= formatting_samples[p95].saturating_mul(4),
        "stack physical key names must improve P95 by at least 20%: formatting={}ns stack={}ns",
        formatting_samples[p95],
        stack_samples[p95],
    );
}

#[test]
fn runtime_interface03_batch68_69_logical_key_name_preserves_formatted_output() {
    for key_code in [0, 1, 9, 10, 13, 32, 48, 65, 90, 91, 999, 65_535, u32::MAX] {
        assert_eq!(
            logical_key_name(key_code),
            logical_key_name_formatting(key_code),
        );
    }
}

#[test]
#[ignore = "release-only stack logical key-name benchmark"]
fn runtime_interface03_batch68_69_stack_logical_key_name_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BUILD_COUNT: usize = 1_000_000;
    const SAMPLE_COUNT: usize = 11;
    const UNKNOWN_KEY_CODE: u32 = u32::MAX;
    let mut formatting_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut stack_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_formatting = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(logical_key_name_formatting(black_box(UNKNOWN_KEY_CODE)));
            }
            started.elapsed().as_nanos()
        };
        let measure_stack = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(logical_key_name(black_box(UNKNOWN_KEY_CODE)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            formatting_samples.push(measure_formatting());
            stack_samples.push(measure_stack());
        } else {
            stack_samples.push(measure_stack());
            formatting_samples.push(measure_formatting());
        }
    }

    formatting_samples.sort_unstable();
    stack_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STACK_LOGICAL_KEY_NAME_BENCH_V1 builds={BUILD_COUNT} digits=10 samples={SAMPLE_COUNT} formatting_p95_ns={} stack_p95_ns={}",
        formatting_samples[p95], stack_samples[p95],
    );
    assert!(
        stack_samples[p95].saturating_mul(5) <= formatting_samples[p95].saturating_mul(4),
        "stack logical key names must improve P95 by at least 20%: formatting={}ns stack={}ns",
        formatting_samples[p95],
        stack_samples[p95],
    );
}
