use super::super::super::super::data::FrameRect;
use super::super::super::{UiProfileFrame, UiProfileNamedFrame};
use super::visibility::{is_visible_frame, is_visible_profile_frame};

pub(in crate::ui::retained_host::host_contract) fn push_named_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: impl Into<String>,
    kind: impl Into<String>,
    surface: impl Into<String>,
    frame: FrameRect,
    clip: Option<FrameRect>,
) {
    if !is_visible_frame(&frame) {
        return;
    }
    push_named_profile_frame(out, id, kind, surface, frame.into(), clip.map(Into::into));
}

pub(in crate::ui::retained_host::host_contract) fn push_named_profile_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: impl Into<String>,
    kind: impl Into<String>,
    surface: impl Into<String>,
    frame: UiProfileFrame,
    clip: Option<UiProfileFrame>,
) {
    if !is_visible_profile_frame(&frame) {
        return;
    }
    out.push(UiProfileNamedFrame {
        id: id.into(),
        kind: kind.into(),
        surface: surface.into(),
        frame,
        clip,
    });
}

#[cfg(test)]
mod optimization_batch_de_editor342_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const IDS_PER_SAMPLE: usize = 16_384;

    #[test]
    fn optimization_batch_de_editor342_named_frame_reuses_an_owned_dynamic_id_buffer() {
        let id = "template.editor.scene.viewport.control-with-a-long-identity".to_string();
        let id_buffer = id.as_ptr();
        let mut frames = Vec::new();

        push_named_profile_frame(
            &mut frames,
            id,
            "template_control",
            "editor.scene",
            UiProfileFrame {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            },
            None,
        );

        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].id.as_ptr(), id_buffer);
        assert_eq!(frames[0].kind, "template_control");
        assert_eq!(frames[0].surface, "editor.scene");
    }

    #[test]
    fn optimization_batch_de_editor342_dynamic_callers_pass_owned_ids_without_a_borrowed_copy() {
        const NAMED: &str = include_str!("named.rs");
        const ACTIVITY_RAIL: &str = include_str!("../pane_frames/activity_rail.rs");
        const SURFACE_CONTROLS: &str = include_str!("../pane_frames/surface_frame/controls.rs");
        const TEMPLATE_NODES: &str = include_str!("../pane_frames/template_nodes.rs");

        let production = NAMED.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("id: impl Into<String>"));
        assert!(production.contains("id: id.into()"));
        for caller in [ACTIVITY_RAIL, SURFACE_CONTROLS, TEMPLATE_NODES] {
            assert!(!caller.contains(".as_str(),"));
        }
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_de_editor342_named_frame_owned_id_p95() {
        let suffixes = (0..IDS_PER_SAMPLE)
            .map(|index| {
                format!(
                    "control-{index:05}-with-stable-editor-profiling-identity-{}",
                    "x".repeat(96)
                )
            })
            .collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&suffixes, false));
                optimized.push(measure(&suffixes, true));
            } else {
                optimized.push(measure(&suffixes, true));
                legacy.push(measure(&suffixes, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR342_NAMED_FRAME_OWNED_ID_BENCH_V1 sample_pairs={SAMPLE_PAIRS} ids_per_sample={IDS_PER_SAMPLE} legacy_id_allocations_per_sample={} optimized_id_allocations_per_sample={IDS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            IDS_PER_SAMPLE * 2,
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "owned dynamic IDs must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure(suffixes: &[String], optimized: bool) -> u128 {
        let started = Instant::now();
        let ids = if optimized {
            suffixes
                .iter()
                .map(|suffix| format!("template.editor.scene.{}", black_box(suffix)))
                .collect::<Vec<_>>()
        } else {
            suffixes
                .iter()
                .map(|suffix| {
                    let formatted = format!("template.editor.scene.{}", black_box(suffix));
                    black_box(formatted.as_str()).to_string()
                })
                .collect::<Vec<_>>()
        };
        black_box(ids);
        started.elapsed().as_nanos().max(1)
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
