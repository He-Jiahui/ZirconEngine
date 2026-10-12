use super::*;

fn color_token_hex_formatting(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    if alpha == u8::MAX {
        format!("#{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
    }
}

#[test]
fn runtime_interface03_batch55_61_color_token_hex_preserves_formatting_output() {
    for (channels, expected) in [
        ([0, 0, 0, 0], "#00000000"),
        ([255, 255, 255, 255], "#ffffff"),
        ([1, 15, 16, 254], "#010f10fe"),
        ([18, 52, 86, 120], "#12345678"),
    ] {
        let color = UiRgbaColor::from_u8(channels[0], channels[1], channels[2], channels[3]);
        assert_eq!(color_token_hex(color), expected);
        assert_eq!(color_token_hex(color), color_token_hex_formatting(color));
    }
}

#[test]
#[ignore = "release-only exact-capacity color token hexadecimal benchmark"]
fn runtime_interface03_batch55_61_stack_color_token_hex_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BUILD_COUNT: usize = 1_000_000;
    const SAMPLE_COUNT: usize = 11;
    let colors = [
        UiRgbaColor::from_u8(0x12, 0x34, 0xab, 0xff),
        UiRgbaColor::from_u8(0x12, 0x34, 0xab, 0xcd),
    ];
    let mut formatting_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut stack_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_formatting = || {
            let started = Instant::now();
            for index in 0..BUILD_COUNT {
                black_box(color_token_hex_formatting(black_box(colors[index & 1])));
            }
            started.elapsed().as_nanos()
        };
        let measure_stack = || {
            let started = Instant::now();
            for index in 0..BUILD_COUNT {
                black_box(color_token_hex(black_box(colors[index & 1])));
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
        "RUNTIME_INTERFACE03_STACK_COLOR_TOKEN_HEX_BENCH_V1 builds={BUILD_COUNT} samples={SAMPLE_COUNT} formatting_p95_ns={} stack_p95_ns={}",
        formatting_samples[p95], stack_samples[p95],
    );
    assert!(
        stack_samples[p95].saturating_mul(5) <= formatting_samples[p95].saturating_mul(4),
        "stack color-token hexadecimal encoding must improve P95 by at least 20%: formatting={}ns stack={}ns",
        formatting_samples[p95],
        stack_samples[p95],
    );
}
