use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchButtonKind;
use super::super::super::template_node_labels::template_node_label;
use super::super::geometry::frame_is_within;
use super::super::layers::label_order;
use super::glyph::{
    button_glyph, button_glyph_width, chevron_width, has_leading_asset_icon, has_leading_glyph,
    has_trailing_chevron, leading_glyph_rect, push_content_asset_icon, push_content_glyph,
    trailing_glyph_rect,
};
use super::layout::button_content_layout;
use super::metrics::{
    button_label_font_size_for_slot, button_label_paint_style, measured_label_ink_width,
};
use super::style::button_content_style;
use super::text::push_button_label;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_button_content(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: WorkbenchButtonKind,
    opacity: f32,
) {
    let label = if node.icon_placement.eq_ignore_ascii_case("icon_only") {
        String::new()
    } else {
        template_node_label(node, None)
    };
    let has_label = !label.trim().is_empty();
    let content_style = button_content_style(node, kind);
    let content_y_offset = content_style.y_offset;
    let glyph = button_glyph(node);
    let glyph_width = button_glyph_width(node, glyph, has_label);
    let chevron_width = chevron_width(glyph);
    let text_style = button_label_paint_style(node, kind);
    let font_size =
        button_label_font_size_for_slot(node, rect, &label, text_style, glyph_width, chevron_width);
    let label_ink_width = measured_label_ink_width(&label, font_size, text_style);
    let layout = button_content_layout(node, rect, glyph_width, chevron_width, label_ink_width);
    let mut x = layout.start_x;

    if has_leading_asset_icon(node) {
        let glyph_rect = offset_rect_y(leading_glyph_rect(rect, x), content_y_offset);
        if frame_is_within(&glyph_rect, rect) {
            let rendered_asset = push_content_asset_icon(
                commands,
                node,
                &glyph_rect,
                clip,
                order,
                content_style.glyph,
                opacity,
            );
            if !rendered_asset && has_leading_glyph(glyph) {
                push_content_glyph(
                    commands,
                    &glyph_rect,
                    clip,
                    order,
                    glyph,
                    content_style.glyph,
                    opacity,
                );
            }
        }
        x += glyph_width;
    } else if has_leading_glyph(glyph) {
        let glyph_rect = offset_rect_y(leading_glyph_rect(rect, x), content_y_offset);
        if frame_is_within(&glyph_rect, rect) {
            push_content_glyph(
                commands,
                &glyph_rect,
                clip,
                order,
                glyph,
                content_style.glyph,
                opacity,
            );
        }
        x += glyph_width;
    }

    if has_label {
        push_button_label(
            commands,
            rect,
            clip,
            label_order(order),
            x,
            content_y_offset,
            layout.text_slot_width,
            font_size,
            text_style,
            label,
            content_style.text,
            opacity,
        );
    }

    if has_trailing_chevron(glyph) {
        let glyph_rect = offset_rect_y(trailing_glyph_rect(rect), content_y_offset);
        if frame_is_within(&glyph_rect, rect) {
            push_content_glyph(
                commands,
                &glyph_rect,
                clip,
                order,
                glyph,
                content_style.glyph,
                opacity,
            );
        }
    }
}

fn offset_rect_y(mut rect: FrameRect, offset: f32) -> FrameRect {
    rect.y += offset;
    rect
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    const LABEL_BYTES: usize = 4_096;
    const CHECKS_PER_SAMPLE: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fx_editor410_button_label_presence_is_computed_once() {
        let source = include_str!("entry.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("button content production source");

        assert_eq!(production.matches("label.trim().is_empty()").count(), 1);
        assert!(production.contains("button_glyph_width(node, glyph, has_label)"));
        assert!(production.contains("if has_label"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fx_editor410_button_label_presence_benchmark() {
        let label = " ".repeat(LABEL_BYTES);
        for _ in 0..4 {
            black_box(measure_checks(&label, false));
            black_box(measure_checks(&label, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(&label, false));
                optimized_samples.push(measure_checks(&label, true));
            } else {
                optimized_samples.push(measure_checks(&label, true));
                legacy_samples.push(measure_checks(&label, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR410_CACHED_BUTTON_LABEL_PRESENCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} label_bytes={LABEL_BYTES} checks_per_sample={CHECKS_PER_SAMPLE} legacy_trim_scans_per_button=2 optimized_trim_scans_per_button=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 65 / 100);
    }

    fn measure_checks(label: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            if optimized {
                let has_label = !black_box(label).trim().is_empty();
                black_box(has_label);
                black_box(has_label);
            } else {
                black_box(!black_box(label).trim().is_empty());
                black_box(!black_box(label).trim().is_empty());
            }
        }
        started.elapsed().as_nanos().max(1)
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
