use super::*;

fn trim_float_allocating(value: f64) -> String {
    let rounded = value.round();
    if (value - rounded).abs() < f64::EPSILON {
        format!("{rounded:.0}")
    } else {
        let mut text = format!("{value:.3}");
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}

fn display_text_allocating(value: &UiValue) -> String {
    match value {
        UiValue::Vec2(value) => format!(
            "{}, {}",
            trim_float_allocating(value[0]),
            trim_float_allocating(value[1])
        ),
        UiValue::Vec3(value) => format!(
            "{}, {}, {}",
            trim_float_allocating(value[0]),
            trim_float_allocating(value[1]),
            trim_float_allocating(value[2])
        ),
        UiValue::Vec4(value) => format!(
            "{}, {}, {}, {}",
            trim_float_allocating(value[0]),
            trim_float_allocating(value[1]),
            trim_float_allocating(value[2]),
            trim_float_allocating(value[3])
        ),
        _ => value.display_text(),
    }
}

#[test]
fn runtime_interface03_batch47_48_single_buffer_vector_display_preserves_allocating_output() {
    for value in [
        UiValue::Vec2([1.0, -2.125]),
        UiValue::Vec3([0.007, 4096.5, -0.0]),
        UiValue::Vec4([f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 12.340]),
    ] {
        assert_eq!(value.display_text(), display_text_allocating(&value));
    }
}

#[test]
#[ignore = "release-only single-buffer value display benchmark"]
fn runtime_interface03_batch47_48_single_buffer_value_display_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const DISPLAY_COUNT: usize = 500_000;
    const SAMPLE_COUNT: usize = 11;
    let value = UiValue::Vec4([12.0, -3.125, 4096.5, 0.007]);
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut single_buffer_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..DISPLAY_COUNT {
                black_box(display_text_allocating(black_box(&value)));
            }
            started.elapsed().as_nanos()
        };
        let measure_single_buffer = || {
            let started = Instant::now();
            for _ in 0..DISPLAY_COUNT {
                black_box(value.display_text());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            single_buffer_samples.push(measure_single_buffer());
        } else {
            single_buffer_samples.push(measure_single_buffer());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    single_buffer_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_BUFFER_VALUE_DISPLAY_BENCH_V1 displays={DISPLAY_COUNT} components=4 samples={SAMPLE_COUNT} allocating_p95_ns={} single_buffer_p95_ns={}",
        allocating_samples[p95], single_buffer_samples[p95],
    );
    assert!(
        single_buffer_samples[p95].saturating_mul(5) <= allocating_samples[p95].saturating_mul(4),
        "single-buffer value display must improve P95 by at least 20%: allocating={}ns single_buffer={}ns",
        allocating_samples[p95],
        single_buffer_samples[p95],
    );
}
