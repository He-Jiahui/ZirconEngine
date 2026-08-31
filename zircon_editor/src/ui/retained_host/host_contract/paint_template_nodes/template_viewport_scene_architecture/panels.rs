use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::border_color;
use super::super::template_viewport_scene_structure::push_base_surface;
use super::primitives::color_with_alpha_factor;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_side_panel_detail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(rect, clip).is_none() {
        return;
    }

    push_base_surface(commands, node, rect, clip, order, opacity);
    let line_color = color_with_alpha_factor(border_color(node), 1.75);
    for y in [rect.y + 36.0, rect.y + 78.0, rect.y + 126.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + 12.0,
                y,
                width: (rect.width - 24.0).max(1.0),
                height: 1.0,
            },
            Some(clip.clone()),
            order + 1,
            Some(line_color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    fn legacy_push_side_panel_detail(
        commands: &mut Vec<HostPaintCommand>,
        node: &TemplatePaneNodeData,
        rect: &FrameRect,
        clip: &FrameRect,
        order: i32,
        opacity: f32,
    ) {
        push_base_surface(commands, node, rect, clip, order, opacity);
        let line_color = color_with_alpha_factor(border_color(node), 1.75);
        for y in [rect.y + 36.0, rect.y + 78.0, rect.y + 126.0] {
            commands.push(HostPaintCommand::quad(
                FrameRect {
                    x: rect.x + 12.0,
                    y,
                    width: (rect.width - 24.0).max(1.0),
                    height: 1.0,
                },
                Some(clip.clone()),
                order + 1,
                Some(line_color),
                None,
                0.0,
                0.0,
                opacity,
            ));
        }
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    #[test]
    fn optimization_batch_hj_editor586_side_panel_clip_preserves_visible_commands() {
        let node = TemplatePaneNodeData::default();
        let rect = FrameRect {
            x: 8.0,
            y: 12.0,
            width: 240.0,
            height: 180.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 240.0,
        };
        let mut legacy = Vec::new();
        let mut optimized = Vec::new();

        legacy_push_side_panel_detail(&mut legacy, &node, &rect, &clip, 17, 0.8);
        push_side_panel_detail(&mut optimized, &node, &rect, &clip, 17, 0.8);

        assert_eq!(optimized.len(), legacy.len());
        for (optimized, legacy) in optimized.iter().zip(&legacy) {
            assert_eq!(optimized.frame, legacy.frame);
            assert_eq!(optimized.clip_frame, legacy.clip_frame);
            assert_eq!(optimized.z_index, legacy.z_index);
            assert_eq!(optimized.background_color, legacy.background_color);
            assert_eq!(optimized.opacity, legacy.opacity);
        }
    }

    #[test]
    fn optimization_batch_hj_editor586_side_panel_clip_rejects_offscreen_commands() {
        let mut commands = Vec::new();
        push_side_panel_detail(
            &mut commands,
            &TemplatePaneNodeData::default(),
            &FrameRect {
                x: 2_000.0,
                y: 2_000.0,
                width: 240.0,
                height: 180.0,
            },
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 240.0,
            },
            0,
            1.0,
        );

        assert!(commands.is_empty());
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_hj_editor586_side_panel_clip_performance_evidence() {
        const SAMPLE_PAIRS: usize = 21;
        const PANELS_PER_SAMPLE: usize = 16_384;

        let node = TemplatePaneNodeData::default();
        let rect = FrameRect {
            x: 2_000.0,
            y: 2_000.0,
            width: 240.0,
            height: 180.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 240.0,
        };
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            let measure_legacy = || {
                let mut commands = Vec::new();
                let started = Instant::now();
                for _ in 0..PANELS_PER_SAMPLE {
                    legacy_push_side_panel_detail(
                        &mut commands,
                        black_box(&node),
                        black_box(&rect),
                        black_box(&clip),
                        0,
                        1.0,
                    );
                }
                black_box(commands.len());
                started.elapsed().as_nanos().max(1)
            };
            let measure_optimized = || {
                let mut commands = Vec::new();
                let started = Instant::now();
                for _ in 0..PANELS_PER_SAMPLE {
                    push_side_panel_detail(
                        &mut commands,
                        black_box(&node),
                        black_box(&rect),
                        black_box(&clip),
                        0,
                        1.0,
                    );
                }
                black_box(commands.len());
                started.elapsed().as_nanos().max(1)
            };

            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "EDITOR586_SIDE_PANEL_CLIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             panels_per_sample={PANELS_PER_SAMPLE} legacy_commands_per_sample={} \
             optimized_commands_per_sample=0 legacy_p95_ns={legacy_p95} \
             optimized_p95_ns={optimized_p95}",
            PANELS_PER_SAMPLE * 4,
        );
        assert!(
            optimized_p95 * 100 <= legacy_p95 * 25,
            "offscreen clip P95 {optimized_p95}ns exceeded 25% of legacy P95 {legacy_p95}ns"
        );
    }
}
