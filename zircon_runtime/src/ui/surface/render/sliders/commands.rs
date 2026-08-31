use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::UiRgbaColor,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
};

use super::{SliderRenderState, SliderVisual};

#[allow(clippy::too_many_arguments)]
pub(super) fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: UiRgbaColor,
    border: Option<UiRgbaColor>,
    border_width: f32,
    corner_radius: f32,
    state: &SliderRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(css_color(background)),
            border_color: border.map(css_color),
            border_width,
            corner_radius,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
    visual: &SliderVisual,
    state: &SliderRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(css_color(foreground)),
            font_size: visual.font_size,
            line_height: visual.line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}

fn css_color(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    let mut value = String::with_capacity(if alpha == u8::MAX { 7 } else { 9 });
    value.push('#');
    push_lower_hex_byte(&mut value, red);
    push_lower_hex_byte(&mut value, green);
    push_lower_hex_byte(&mut value, blue);
    if alpha != u8::MAX {
        push_lower_hex_byte(&mut value, alpha);
    }
    value
}

fn push_lower_hex_byte(output: &mut String, value: u8) {
    const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";
    output.push(char::from(LOWER_HEX[usize::from(value >> 4)]));
    output.push(char::from(LOWER_HEX[usize::from(value & 0x0f)]));
}

#[cfg(test)]
mod optimization_batch_et_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

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

    #[test]
    fn optimization_batch_et_slider_css_color_preserves_rgba_encoding() {
        for color in [
            UiRgbaColor::from_u8(0, 0, 0, 255),
            UiRgbaColor::from_u8(12, 34, 56, 78),
            UiRgbaColor::from_u8(255, 160, 1, 0),
        ] {
            assert_eq!(css_color(color), legacy_css_color(color));
        }
        assert_eq!(css_color(UiRgbaColor::from_u8(12, 34, 56, 78)), "#0c22384e");
    }

    #[test]
    #[ignore = "release-only direct CSS hex encoding benchmark"]
    fn optimization_batch_et_slider_css_color_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const ENCODINGS_PER_SAMPLE: usize = 65_536;

        fn measure(colors: &[UiRgbaColor], encode: fn(UiRgbaColor) -> String) -> u128 {
            let started = Instant::now();
            let mut checksum = 0_usize;
            for index in 0..ENCODINGS_PER_SAMPLE {
                checksum =
                    checksum.wrapping_add(encode(black_box(colors[index % colors.len()])).len());
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

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let colors = [
            UiRgbaColor::from_u8(12, 34, 56, 78),
            UiRgbaColor::from_u8(90, 123, 210, 255),
            UiRgbaColor::from_u8(255, 160, 1, 0),
            UiRgbaColor::from_u8(31, 63, 127, 191),
        ];
        for color in colors {
            assert_eq!(css_color(color), legacy_css_color(color));
        }
        for _ in 0..4 {
            black_box(measure(&colors, legacy_css_color));
            black_box(measure(&colors, css_color));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure(&colors, legacy_css_color));
                optimized_samples.push(measure(&colors, css_color));
            } else {
                optimized_samples.push(measure(&colors, css_color));
                legacy_samples.push(measure(&colors, legacy_css_color));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME452_DIRECT_CSS_HEX_ENCODING_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             encodings_per_sample={ENCODINGS_PER_SAMPLE} color_count={} \
             pair_order=alternating_legacy_even legacy_format_calls_per_encoding=1 \
             legacy_front_inserts_per_encoding=1 optimized_format_calls_per_encoding=0 \
             optimized_front_inserts_per_encoding=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            colors.len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(55),
            "direct CSS hex encoding must reduce P95 by at least 45%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
