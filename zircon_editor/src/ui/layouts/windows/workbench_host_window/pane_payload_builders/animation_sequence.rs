use crate::ui::animation_editor::AnimationEditorPanePresentation;

use super::super::pane_payload::{AnimationSequencePanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    PanePayload::AnimationSequenceV1(animation_sequence_payload(context.animation_pane))
}

pub(super) fn animation_sequence_payload(
    animation: Option<&AnimationEditorPanePresentation>,
) -> AnimationSequencePanePayload {
    let Some(animation) = animation else {
        return AnimationSequencePanePayload {
            mode: String::new(),
            asset_path: String::new(),
            status: String::new(),
            selection: String::new(),
            current_frame: 0,
            timeline_start_frame: 0,
            timeline_end_frame: 0,
            playback_label: String::new(),
            track_items: Vec::new(),
        };
    };
    AnimationSequencePanePayload {
        mode: animation.mode.clone(),
        asset_path: animation.asset_path.clone(),
        status: animation.status.clone(),
        selection: animation.selection_summary.clone(),
        current_frame: animation.current_frame,
        timeline_start_frame: animation.timeline_start_frame,
        timeline_end_frame: animation.timeline_end_frame,
        playback_label: animation.playback_label.clone(),
        track_items: animation.track_items.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::super::animation_graph::animation_graph_payload;
    use super::*;
    use crate::ui::layouts::windows::workbench_host_window::pane_payload::AnimationGraphPanePayload;

    #[test]
    fn optimization_batch_du_selective_animation_sequence_projection_preserves_fields() {
        let presentation = animation_presentation_fixture(2);

        let payload = animation_sequence_payload(Some(&presentation));

        assert_eq!(payload.mode, presentation.mode);
        assert_eq!(payload.asset_path, presentation.asset_path);
        assert_eq!(payload.status, presentation.status);
        assert_eq!(payload.selection, presentation.selection_summary);
        assert_eq!(payload.current_frame, presentation.current_frame);
        assert_eq!(
            payload.timeline_start_frame,
            presentation.timeline_start_frame
        );
        assert_eq!(payload.timeline_end_frame, presentation.timeline_end_frame);
        assert_eq!(payload.playback_label, presentation.playback_label);
        assert_eq!(payload.track_items, presentation.track_items);
    }

    #[test]
    fn optimization_batch_du_selective_animation_sequence_projection_defaults_when_absent() {
        let payload = animation_sequence_payload(None);

        assert!(payload.mode.is_empty());
        assert!(payload.asset_path.is_empty());
        assert!(payload.status.is_empty());
        assert!(payload.selection.is_empty());
        assert_eq!(payload.current_frame, 0);
        assert_eq!(payload.timeline_start_frame, 0);
        assert_eq!(payload.timeline_end_frame, 0);
        assert!(payload.playback_label.is_empty());
        assert!(payload.track_items.is_empty());
    }

    #[test]
    fn optimization_batch_du_animation_builders_avoid_full_presentation_clones() {
        for production in [
            include_str!("animation_sequence.rs"),
            include_str!("animation_graph.rs"),
        ] {
            let production = production
                .split("#[cfg(test)]")
                .next()
                .expect("animation payload builder production source");
            assert!(!production.contains("animation_pane.cloned()"));
            assert!(!production.contains("AnimationEditorPanePresentation::default"));
        }
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_du_selective_animation_pane_projection_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const PROJECTION_PAIRS_PER_SAMPLE: usize = 128;
        const ITEMS_PER_MODE: usize = 256;

        let presentation = animation_presentation_fixture(ITEMS_PER_MODE);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_animation_projections(
                    &presentation,
                    PROJECTION_PAIRS_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_animation_projections(
                    &presentation,
                    PROJECTION_PAIRS_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_animation_projections(
                    &presentation,
                    PROJECTION_PAIRS_PER_SAMPLE,
                    true,
                ));
                legacy_samples.push(measure_animation_projections(
                    &presentation,
                    PROJECTION_PAIRS_PER_SAMPLE,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR358_SELECTIVE_ANIMATION_PANE_PROJECTION_BENCH_V1 projection_pairs_per_sample={PROJECTION_PAIRS_PER_SAMPLE} items_per_mode={ITEMS_PER_MODE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "selective animation pane projection p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn animation_presentation_fixture(item_count: usize) -> AnimationEditorPanePresentation {
        let items = |prefix: &str| {
            (0..item_count)
                .map(|index| format!("{prefix}_{index:04}_{}", "payload".repeat(16)))
                .collect::<Vec<_>>()
        };
        AnimationEditorPanePresentation {
            mode: "animation".to_owned(),
            asset_path: "project://animations/hero.animation".to_owned(),
            status: "ready".to_owned(),
            selection_summary: "Root/Hips".to_owned(),
            current_frame: 42,
            timeline_start_frame: 10,
            timeline_end_frame: 240,
            playback_label: "Playing".to_owned(),
            track_items: items("track"),
            parameter_items: items("parameter"),
            node_items: items("node"),
            state_items: items("state"),
            transition_items: items("transition"),
        }
    }

    fn measure_animation_projections(
        presentation: &AnimationEditorPanePresentation,
        projection_pairs: usize,
        optimized: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..projection_pairs {
            let (sequence, graph) = if optimized {
                (
                    animation_sequence_payload(Some(presentation)),
                    animation_graph_payload(Some(presentation)),
                )
            } else {
                (
                    legacy_animation_sequence_payload(presentation),
                    legacy_animation_graph_payload(presentation),
                )
            };
            checksum = checksum
                .wrapping_add(sequence.track_items.len())
                .wrapping_add(graph.parameter_items.len())
                .wrapping_add(graph.node_items.len())
                .wrapping_add(graph.state_items.len())
                .wrapping_add(graph.transition_items.len());
            black_box((sequence, graph));
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn legacy_animation_sequence_payload(
        presentation: &AnimationEditorPanePresentation,
    ) -> AnimationSequencePanePayload {
        let animation = presentation.clone();
        AnimationSequencePanePayload {
            mode: animation.mode,
            asset_path: animation.asset_path,
            status: animation.status,
            selection: animation.selection_summary,
            current_frame: animation.current_frame,
            timeline_start_frame: animation.timeline_start_frame,
            timeline_end_frame: animation.timeline_end_frame,
            playback_label: animation.playback_label,
            track_items: animation.track_items,
        }
    }

    fn legacy_animation_graph_payload(
        presentation: &AnimationEditorPanePresentation,
    ) -> AnimationGraphPanePayload {
        let animation = presentation.clone();
        AnimationGraphPanePayload {
            mode: animation.mode,
            asset_path: animation.asset_path,
            status: animation.status,
            selection: animation.selection_summary,
            parameter_items: animation.parameter_items,
            node_items: animation.node_items,
            state_items: animation.state_items,
            transition_items: animation.transition_items,
        }
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
