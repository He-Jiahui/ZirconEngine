use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;

mod ripple;
mod state;

use ripple::{push_ripple_commands, ripple_is_visible};
use state::{state_layer_color, state_layer_opacity};

#[cfg(test)]
use ripple::{ripple_diameter, ripple_rect, RIPPLE_DIAMETER_EXPANSION};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_state_layer_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    corner_radius: f32,
    order: i32,
    opacity_multiplier: f32,
) {
    let overlay_opacity = state_layer_opacity(node);
    let ripple_visible = ripple_is_visible(node);
    if overlay_opacity.is_none() && !ripple_visible {
        return;
    }
    if !ripple_visible && intersect(rect, clip).is_none() {
        return;
    }

    let color = state_layer_color(node);
    if let Some(opacity) = overlay_opacity {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            corner_radius,
            opacity * opacity_multiplier,
        ));
    }

    if ripple_visible {
        push_ripple_commands(
            commands,
            node,
            rect,
            clip,
            order + 1,
            color,
            opacity_multiplier,
        );
    }
}

#[cfg(test)]
#[path = "material_state_layer_tests/mod.rs"]
mod tests;

#[cfg(test)]
mod optimization_batch_hb_editor583_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_hb_editor583_offscreen_overlay_emits_no_command() {
        let node = hovered_overlay();
        let rect = offscreen_rect();
        let clip = visible_clip();
        let mut commands = Vec::new();

        push_state_layer_commands(&mut commands, &node, &rect, &clip, 4.0, 1, 1.0);

        assert!(commands.is_empty());
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_hb_editor583_state_overlay_clip_early_exit_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 65_536;
        let node = hovered_overlay();
        let rect = offscreen_rect();
        let clip = visible_clip();
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
            "EDITOR583_STATE_OVERLAY_CLIP_EARLY_EXIT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "state-overlay clip early exit must improve offscreen P95 by at least 50%"
        );
    }

    fn measure(
        optimized: bool,
        node: &TemplatePaneNodeData,
        rect: &FrameRect,
        clip: &FrameRect,
        iterations: usize,
    ) -> u128 {
        let started = Instant::now();
        let mut commands = Vec::new();
        for _ in 0..iterations {
            if optimized {
                push_state_layer_commands(&mut commands, node, rect, clip, 4.0, 1, 1.0);
            } else {
                push_state_layer_overlay_legacy(&mut commands, node, rect, clip);
            }
            commands.clear();
        }
        black_box(commands.len());
        started.elapsed().as_nanos().max(1)
    }

    fn push_state_layer_overlay_legacy(
        commands: &mut Vec<HostPaintCommand>,
        node: &TemplatePaneNodeData,
        rect: &FrameRect,
        clip: &FrameRect,
    ) {
        let opacity = state_layer_opacity(node).expect("hovered fixture should emit an overlay");
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            1,
            Some(state_layer_color(node)),
            None,
            0.0,
            4.0,
            opacity,
        ));
    }

    fn hovered_overlay() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            state_layer_enabled: true,
            hovered: true,
            ..TemplatePaneNodeData::default()
        }
    }

    fn offscreen_rect() -> FrameRect {
        FrameRect {
            x: 1000.0,
            y: 1000.0,
            width: 100.0,
            height: 40.0,
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
