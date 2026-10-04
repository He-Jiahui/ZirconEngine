use super::*;

fn fixed_float_array_allocating<const N: usize>(value: &TomlValue) -> Option<[f64; N]> {
    let values = value.as_array()?;
    if values.len() != N {
        return None;
    }
    let floats = values
        .iter()
        .map(|value| match value {
            TomlValue::Integer(value) => Some(*value as f64),
            TomlValue::Float(value) => Some(*value),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    floats.try_into().ok()
}

#[test]
fn runtime_interface03_batch47_48_stack_fixed_float_array_preserves_allocating_results() {
    let valid_two = TomlValue::Array(vec![TomlValue::Integer(1), TomlValue::Float(2.5)]);
    let valid_three = TomlValue::Array(vec![
        TomlValue::Float(-1.25),
        TomlValue::Integer(2),
        TomlValue::Float(3.75),
    ]);
    let valid_four = TomlValue::Array(vec![
        TomlValue::Integer(1),
        TomlValue::Integer(2),
        TomlValue::Integer(3),
        TomlValue::Integer(4),
    ]);
    assert_eq!(
        fixed_float_array::<2>(&valid_two),
        fixed_float_array_allocating::<2>(&valid_two)
    );
    assert_eq!(
        fixed_float_array::<3>(&valid_three),
        fixed_float_array_allocating::<3>(&valid_three)
    );
    assert_eq!(
        fixed_float_array::<4>(&valid_four),
        fixed_float_array_allocating::<4>(&valid_four)
    );

    for invalid in [
        TomlValue::Array(vec![TomlValue::Integer(1)]),
        TomlValue::Array(vec![
            TomlValue::Integer(1),
            TomlValue::Integer(2),
            TomlValue::Integer(3),
        ]),
        TomlValue::Array(vec![
            TomlValue::Integer(1),
            TomlValue::String("bad".to_string()),
        ]),
        TomlValue::Array(vec![TomlValue::Boolean(true), TomlValue::Integer(2)]),
        TomlValue::String("not-an-array".to_string()),
    ] {
        assert_eq!(
            fixed_float_array::<2>(&invalid),
            fixed_float_array_allocating::<2>(&invalid)
        );
    }
}

#[test]
#[ignore = "release-only stack fixed float array benchmark"]
fn runtime_interface03_batch47_48_stack_fixed_float_array_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const CONVERSION_COUNT: usize = 1_000_000;
    const SAMPLE_COUNT: usize = 11;
    let value = TomlValue::Array(vec![
        TomlValue::Integer(1),
        TomlValue::Float(-2.5),
        TomlValue::Integer(3),
        TomlValue::Float(4.75),
    ]);
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut stack_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..CONVERSION_COUNT {
                black_box(fixed_float_array_allocating::<4>(black_box(&value)));
            }
            started.elapsed().as_nanos()
        };
        let measure_stack = || {
            let started = Instant::now();
            for _ in 0..CONVERSION_COUNT {
                black_box(fixed_float_array::<4>(black_box(&value)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            stack_samples.push(measure_stack());
        } else {
            stack_samples.push(measure_stack());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    stack_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STACK_FIXED_FLOAT_ARRAY_BENCH_V1 conversions={CONVERSION_COUNT} components=4 samples={SAMPLE_COUNT} allocating_p95_ns={} stack_p95_ns={}",
        allocating_samples[p95], stack_samples[p95],
    );
    assert!(
        stack_samples[p95].saturating_mul(5) <= allocating_samples[p95].saturating_mul(4),
        "stack fixed float array must improve P95 by at least 20%: allocating={}ns stack={}ns",
        allocating_samples[p95],
        stack_samples[p95],
    );
}
