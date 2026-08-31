use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_geometry::shadow_check_rect;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_geometry::is_paintable_rect;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_glyphs::push_inspector_check_tick;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_kind::bool_value;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;

use super::super::primitives::push_nested_label;
use super::super::style::{inspector_row_palette, resource_glyph_color, resource_label_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shadow_check_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let palette = inspector_row_palette();
    let metrics = current_host_metrics();
    push_nested_label(
        commands,
        rect,
        clip,
        order,
        node.text.trim(),
        resource_label_color(node),
        opacity,
    );
    let check = shadow_check_rect(node, rect);
    if !is_paintable_rect(&check) {
        return;
    }
    let checked = bool_value(&node.value_text) || node.checked || node.selected;
    commands.push(HostPaintCommand::quad(
        check.clone(),
        Some(clip.clone()),
        order + 1,
        Some(if checked {
            palette.checked_surface
        } else {
            palette.field_surface
        }),
        Some(if checked {
            palette.checked_border
        } else {
            palette.field_border
        }),
        metrics.border_width,
        metrics.radius_control,
        opacity,
    ));
    if checked {
        push_inspector_check_tick(
            commands,
            &check,
            clip,
            order + 2,
            resource_glyph_color(node),
            opacity,
        );
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::bool_value;

    const INPUT: &str = "checked";
    const CHECKS_PER_SAMPLE: usize = 1_048_576;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_ga_editor413_boolean_paint_uses_the_parser_trim_once() {
        let check_source = include_str!("check.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("shadow check production source");
        let select_source = include_str!("select.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("shadow select production source");

        assert!(check_source.contains("bool_value(&node.value_text)"));
        assert!(select_source.contains("bool_display_value(&node.value_text)"));
        assert!(!check_source.contains("bool_value(node.value_text.trim())"));
        assert!(!select_source.contains("bool_display_value(node.value_text.trim())"));
        assert!(bool_value(" \t checked \t "));
        assert!(!bool_value(" \t unchecked \t "));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ga_editor413_boolean_paint_dispatch_benchmark() {
        for _ in 0..4 {
            black_box(measure_checks(INPUT, false));
            black_box(measure_checks(INPUT, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(INPUT, false));
                optimized_samples.push(measure_checks(INPUT, true));
            } else {
                optimized_samples.push(measure_checks(INPUT, true));
                legacy_samples.push(measure_checks(INPUT, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR413_BOOLEAN_PAINT_DISPATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_trim_calls_per_check=2 optimized_trim_calls_per_check=1 legacy_candidate_comparisons_per_check=5 optimized_candidate_comparisons_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
            INPUT.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 65 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let checked = if optimized {
                bool_value(black_box(input))
            } else {
                legacy_bool_value(black_box(input).trim())
            };
            black_box(checked);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_bool_value(value: &str) -> bool {
        let value = value.trim();
        value == "1"
            || ["true", "on", "yes", "check", "checked"]
                .iter()
                .any(|candidate| value.eq_ignore_ascii_case(candidate))
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
