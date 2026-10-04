use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::style::UiRgbaColor;

use super::css_color;

#[test]
fn optimization_batch_20260831gr_runtime573_css_color_preserves_hex_forms() {
    assert_eq!(css_color(UiRgbaColor::from_u8(0, 16, 255, 255)), "#0010ff");
    assert_eq!(css_color(UiRgbaColor::from_u8(1, 35, 69, 127)), "#0123457f");
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260831gr_runtime573_css_color_single_buffer_benchmark() {
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 250_000;
    let colors = [
        UiRgbaColor::from_u8(0, 16, 255, 255),
        UiRgbaColor::from_u8(1, 35, 69, 127),
        UiRgbaColor::from_u8(240, 128, 7, 255),
        UiRgbaColor::from_u8(255, 0, 16, 64),
    ];
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut checksum = 0usize;
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            let (elapsed, value) = measure(ITERATIONS, &colors, legacy_css_color);
            legacy_ns.push(elapsed);
            checksum ^= value;
            let (elapsed, value) = measure(ITERATIONS, &colors, css_color);
            optimized_ns.push(elapsed);
            checksum ^= value;
        } else {
            let (elapsed, value) = measure(ITERATIONS, &colors, css_color);
            optimized_ns.push(elapsed);
            checksum ^= value;
            let (elapsed, value) = measure(ITERATIONS, &colors, legacy_css_color);
            legacy_ns.push(elapsed);
            checksum ^= value;
        }
    }
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
        "single-buffer CSS color P95 must be at least 15% below format/insert: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "RUNTIME573_CSS_COLOR_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} checksum={checksum} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );

    fn measure(
        iterations: usize,
        colors: &[UiRgbaColor],
        operation: fn(UiRgbaColor) -> String,
    ) -> (u128, usize) {
        let started = Instant::now();
        let mut checksum = 0usize;
        for index in 0..iterations {
            checksum =
                checksum.wrapping_add(operation(black_box(colors[index % colors.len()])).len());
        }
        (started.elapsed().as_nanos(), black_box(checksum))
    }

    fn legacy_css_color(color: UiRgbaColor) -> String {
        let [red, green, blue, alpha] = color.to_u8();
        let mut value = if alpha == u8::MAX {
            format!("{red:02x}{green:02x}{blue:02x}")
        } else {
            format!("{red:02x}{green:02x}{blue:02x}{alpha:02x}")
        };
        value.insert(0, '#');
        value
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = (ordered.len() * percentile).div_ceil(100).max(1);
        ordered[rank - 1]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
