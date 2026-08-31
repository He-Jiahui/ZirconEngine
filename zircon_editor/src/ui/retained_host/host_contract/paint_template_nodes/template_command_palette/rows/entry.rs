use super::super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::layers::{row_label_order, row_match_indicator_order};
use super::detail::push_command_row_detail;
use super::indicator::push_command_row_match_indicator;
use super::label::push_command_row_label;
use super::style::command_row_style;
use super::surface::push_command_row_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    option: &TemplatePaneOptionData,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = command_row_style(option);
    push_command_row_surface(commands, row_rect, clip, order, style, opacity);
    push_command_row_match_indicator(
        commands,
        option,
        row_rect,
        clip,
        row_match_indicator_order(order),
        opacity,
    );
    push_command_row_label(
        commands,
        row_rect,
        clip,
        row_label_order(order),
        option.label.as_str(),
        style.text,
        opacity,
    );
    push_command_row_detail(
        commands,
        row_rect,
        clip,
        row_label_order(order),
        option.description.as_str(),
        style.shortcut,
        opacity,
    );
}

#[cfg(test)]
mod optimization_batch_gy_editor580_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_gy_editor580_command_row_text_preserves_culling() {
        let row = offscreen_row();
        let clip = visible_clip();
        let mut commands = Vec::new();

        push_command_row_label(
            &mut commands,
            &row,
            &clip,
            1,
            "Build Project",
            [255; 4],
            1.0,
        );
        push_command_row_detail(&mut commands, &row, &clip, 1, "Ctrl+B", [255; 4], 1.0);

        assert!(commands.is_empty());
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gy_editor580_command_row_text_deferred_clone_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 65_536;
        let text = "offscreen-command-row-text/".repeat(64);
        let row = offscreen_row();
        let clip = visible_clip();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &text, &row, &clip, ITERATIONS));
                optimized.push(measure(true, &text, &row, &clip, ITERATIONS));
            } else {
                optimized.push(measure(true, &text, &row, &clip, ITERATIONS));
                legacy.push(measure(false, &text, &row, &clip, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR580_COMMAND_ROW_DEFERRED_TEXT_CLONE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} text_bytes={} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            text.len(),
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "deferred row text cloning must improve offscreen P95 by at least 50%"
        );
    }

    fn measure(
        optimized: bool,
        text: &str,
        row: &FrameRect,
        clip: &FrameRect,
        iterations: usize,
    ) -> u128 {
        let started = Instant::now();
        let mut commands = Vec::new();
        for _ in 0..iterations {
            if optimized {
                push_text_pair(&mut commands, row, clip, black_box(text));
            } else {
                let label = black_box(text).to_string();
                let detail = black_box(text).to_string();
                push_command_row_label(
                    &mut commands,
                    row,
                    clip,
                    1,
                    &label,
                    [255, 255, 255, 255],
                    1.0,
                );
                push_command_row_detail(
                    &mut commands,
                    row,
                    clip,
                    1,
                    &detail,
                    [255, 255, 255, 255],
                    1.0,
                );
                black_box((&label, &detail));
            }
        }
        black_box(commands.len());
        started.elapsed().as_nanos().max(1)
    }

    fn push_text_pair(
        commands: &mut Vec<HostPaintCommand>,
        row: &FrameRect,
        clip: &FrameRect,
        text: &str,
    ) {
        push_command_row_label(commands, row, clip, 1, text, [255; 4], 1.0);
        push_command_row_detail(commands, row, clip, 1, text, [255; 4], 1.0);
    }

    fn offscreen_row() -> FrameRect {
        FrameRect {
            x: 0.0,
            y: 200.0,
            width: 100.0,
            height: 20.0,
        }
    }

    fn visible_clip() -> FrameRect {
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
