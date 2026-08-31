use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{layout, style};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_preview_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    preview_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: style::DragOverlayPalette,
    metrics: &layout::DragOverlayMetrics,
) {
    let Some(label) = preview_label(node) else {
        return;
    };
    let text_rect = layout::preview_text_frame(preview_rect, metrics);
    if text_rect.width <= 0.0 || text_rect.height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label.to_string(),
        palette.preview_text,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn preview_label(node: &TemplatePaneNodeData) -> Option<&str> {
    [
        node.drag_payload_label.as_str(),
        node.text.as_str(),
        node.drag_payload_reference.as_str(),
        node.value_text.as_str(),
    ]
    .into_iter()
    .map(str::trim)
    .find(|value| !value.is_empty())
}

#[cfg(test)]
mod optimization_batch_gu_editor576_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 31;
    const ITERATIONS: usize = 100_000;

    #[test]
    fn optimization_batch_gu_editor576_preview_label_preserves_priority_and_trimming() {
        let mut node = TemplatePaneNodeData {
            drag_payload_label: "  Dragged asset  ".to_string(),
            text: "Fallback text".to_string(),
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(preview_label(&node), Some("Dragged asset"));
        node.drag_payload_label.clear();
        assert_eq!(preview_label(&node), Some("Fallback text"));

        let production = include_str!("text.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let geometry_gate = production
            .find("if text_rect.width <= 0.0")
            .expect("geometry gate");
        let allocation = production
            .find("label.to_string()")
            .expect("label allocation");
        assert!(geometry_gate < allocation);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gu_editor576_deferred_drag_label_allocation_p95() {
        let node = TemplatePaneNodeData {
            drag_payload_label:
                "  res://environment/industrial/props/large-crate-material-instance.zmaterial  "
                    .to_string(),
            ..TemplatePaneNodeData::default()
        };
        let preview_rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: black_box(0.0),
            height: 24.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 640.0,
            height: 480.0,
        };
        let palette = style::drag_overlay_palette();
        let metrics = layout::drag_overlay_metrics();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&node, &preview_rect, &metrics));
                optimized_samples.push(measure_optimized(
                    &node,
                    &preview_rect,
                    &clip,
                    palette,
                    &metrics,
                ));
            } else {
                optimized_samples.push(measure_optimized(
                    &node,
                    &preview_rect,
                    &clip,
                    palette,
                    &metrics,
                ));
                legacy_samples.push(measure_legacy(&node, &preview_rect, &metrics));
            }
        }

        let legacy_p95_ns = p95(&mut legacy_samples);
        let optimized_p95_ns = p95(&mut optimized_samples);
        println!(
            "EDITOR576_DEFERRED_DRAG_LABEL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} legacy_allocations_per_sample={ITERATIONS} optimized_allocations_per_sample=0 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "expected deferred label allocation to lower p95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure_legacy(
        node: &TemplatePaneNodeData,
        preview_rect: &FrameRect,
        metrics: &layout::DragOverlayMetrics,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..ITERATIONS {
            let Some(label) = preview_label(node).map(str::to_string) else {
                continue;
            };
            let text_rect = layout::preview_text_frame(preview_rect, metrics);
            if text_rect.width <= 0.0 || text_rect.height <= 0.0 {
                black_box(label);
                continue;
            }
        }
        started.elapsed().as_nanos()
    }

    fn measure_optimized(
        node: &TemplatePaneNodeData,
        preview_rect: &FrameRect,
        clip: &FrameRect,
        palette: style::DragOverlayPalette,
        metrics: &layout::DragOverlayMetrics,
    ) -> u128 {
        let started = Instant::now();
        let mut commands = Vec::new();
        for _ in 0..ITERATIONS {
            push_preview_label(
                &mut commands,
                node,
                preview_rect,
                clip,
                0,
                1.0,
                palette,
                metrics,
            );
        }
        black_box(commands.len());
        started.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[samples.len() * 95 / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
