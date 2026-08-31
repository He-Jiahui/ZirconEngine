mod surface;
mod text;

use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::badge_overlay_frame;
use super::super::identity::{badge_is_dot, badge_is_invisible};
use super::super::labels::badge_display_text;
use surface::push_badge_overlay_surface;
use text::push_badge_overlay_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_overlay(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if badge_is_invisible(node) {
        return;
    }
    let display = badge_display_text(node);
    let dot = badge_is_dot(node);
    if !dot && display.is_empty() {
        return;
    }
    let badge_rect = badge_overlay_frame(node, rect, &display, dot);
    if !badge_rect.x.is_finite()
        || !badge_rect.y.is_finite()
        || badge_rect.width <= 0.0
        || badge_rect.height <= 0.0
    {
        return;
    }
    if intersect(&badge_rect, clip).is_none() {
        return;
    }
    push_badge_overlay_surface(
        commands,
        node,
        badge_rect.clone(),
        clip,
        order,
        dot,
        opacity,
    );
    if !dot {
        push_badge_overlay_text(
            commands,
            node,
            &display,
            &badge_rect,
            clip,
            order + 1,
            opacity,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_badge_anchor_does_not_emit_paint_commands() {
        let node = TemplatePaneNodeData {
            component_role: "badge".to_owned(),
            value_text: "1".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: f32::INFINITY,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };
        let mut commands = Vec::new();

        push_badge_overlay(&mut commands, &node, &rect, &rect, 0, 1.0);

        assert!(commands.is_empty());
    }

    #[test]
    fn optimization_batch_gz_editor581_offscreen_badge_emits_no_commands() {
        let node = TemplatePaneNodeData {
            component_role: "badge".to_owned(),
            value_text: "99".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };
        let clip = FrameRect {
            x: 1000.0,
            y: 1000.0,
            width: 10.0,
            height: 10.0,
        };
        let mut commands = Vec::new();

        push_badge_overlay(&mut commands, &node, &rect, &clip, 0, 1.0);

        assert!(commands.is_empty());
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gz_editor581_badge_overlay_clip_early_exit_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 8_192;
        let node = TemplatePaneNodeData {
            component_role: "badge".to_owned(),
            value_text: "99".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };
        let clip = FrameRect {
            x: 1000.0,
            y: 1000.0,
            width: 10.0,
            height: 10.0,
        };
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &node, &rect, &clip, ITERATIONS));
                optimized.push(measure(true, &node, &rect, &clip, ITERATIONS));
            } else {
                optimized.push(measure(true, &node, &rect, &clip, ITERATIONS));
                legacy.push(measure(false, &node, &rect, &clip, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR581_BADGE_OVERLAY_CLIP_EARLY_EXIT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "badge clip early exit must improve offscreen P95 by at least 50%"
        );
    }

    fn measure(
        optimized: bool,
        node: &TemplatePaneNodeData,
        rect: &FrameRect,
        clip: &FrameRect,
        iterations: usize,
    ) -> u128 {
        let started = std::time::Instant::now();
        let mut commands = Vec::new();
        for _ in 0..iterations {
            if optimized {
                push_badge_overlay(&mut commands, node, rect, clip, 0, 1.0);
            } else {
                push_badge_overlay_legacy(&mut commands, node, rect, clip, 0, 1.0);
            }
            commands.clear();
        }
        std::hint::black_box(commands.len());
        started.elapsed().as_nanos().max(1)
    }

    fn push_badge_overlay_legacy(
        commands: &mut Vec<HostPaintCommand>,
        node: &TemplatePaneNodeData,
        rect: &FrameRect,
        clip: &FrameRect,
        order: i32,
        opacity: f32,
    ) {
        if badge_is_invisible(node) {
            return;
        }
        let display = badge_display_text(node);
        let dot = badge_is_dot(node);
        if !dot && display.is_empty() {
            return;
        }
        let badge_rect = badge_overlay_frame(node, rect, display, dot);
        if !badge_rect.x.is_finite()
            || !badge_rect.y.is_finite()
            || badge_rect.width <= 0.0
            || badge_rect.height <= 0.0
        {
            return;
        }
        push_badge_overlay_surface(
            commands,
            node,
            badge_rect.clone(),
            clip,
            order,
            dot,
            opacity,
        );
        if !dot {
            push_badge_overlay_text(
                commands,
                node,
                display,
                &badge_rect,
                clip,
                order + 1,
                opacity,
            );
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
