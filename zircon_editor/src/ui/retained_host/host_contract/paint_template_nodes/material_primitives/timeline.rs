mod connector;
mod dot;
mod geometry;
mod identity;
mod style;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use connector::push_timeline_connector;
use dot::push_timeline_dot;
use identity::{timeline_primitive_kind, TimelinePrimitiveKind};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_timeline_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = timeline_primitive_kind(node) else {
        return false;
    };
    if intersect(rect, clip).is_none() {
        return true;
    }
    match kind {
        TimelinePrimitiveKind::Dot => {
            push_timeline_dot(commands, node, rect, clip, order, opacity);
        }
        TimelinePrimitiveKind::Connector => {
            push_timeline_connector(commands, node, rect, clip, order, opacity);
        }
        TimelinePrimitiveKind::Separator => {}
    }
    true
}

#[cfg(test)]
mod optimization_batch_ha_editor582_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_ha_editor582_offscreen_timeline_preserves_routing() {
        let node = timeline_dot();
        let rect = offscreen_rect();
        let clip = visible_clip();
        let mut commands = Vec::new();

        assert!(push_timeline_primitive_commands(
            &mut commands,
            &node,
            &rect,
            &clip,
            1,
            1.0,
        ));
        assert!(commands.is_empty());

        let unknown = TemplatePaneNodeData::default();
        assert!(!push_timeline_primitive_commands(
            &mut commands,
            &unknown,
            &rect,
            &clip,
            1,
            1.0,
        ));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_ha_editor582_timeline_clip_early_exit_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 65_536;
        let node = timeline_dot();
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
            "EDITOR582_TIMELINE_CLIP_EARLY_EXIT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "timeline clip early exit must improve offscreen P95 by at least 50%"
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
                black_box(push_timeline_primitive_commands(
                    &mut commands,
                    black_box(node),
                    rect,
                    clip,
                    1,
                    1.0,
                ));
            } else {
                black_box(push_timeline_primitive_commands_legacy(
                    &mut commands,
                    black_box(node),
                    rect,
                    clip,
                    1,
                    1.0,
                ));
            }
            commands.clear();
        }
        black_box(commands.len());
        started.elapsed().as_nanos().max(1)
    }

    fn push_timeline_primitive_commands_legacy(
        commands: &mut Vec<HostPaintCommand>,
        node: &TemplatePaneNodeData,
        rect: &FrameRect,
        clip: &FrameRect,
        order: i32,
        opacity: f32,
    ) -> bool {
        match timeline_primitive_kind(node) {
            Some(TimelinePrimitiveKind::Dot) => {
                push_timeline_dot(commands, node, rect, clip, order, opacity);
            }
            Some(TimelinePrimitiveKind::Connector) => {
                push_timeline_connector(commands, node, rect, clip, order, opacity);
            }
            Some(TimelinePrimitiveKind::Separator) => {}
            None => return false,
        }
        true
    }

    fn timeline_dot() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            component_role: "timeline-dot".to_owned(),
            ..TemplatePaneNodeData::default()
        }
    }

    fn offscreen_rect() -> FrameRect {
        FrameRect {
            x: 1000.0,
            y: 1000.0,
            width: 24.0,
            height: 24.0,
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
