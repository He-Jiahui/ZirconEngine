use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    AnimationEditorPaneViewData, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::template_runtime::EditorUiHostRuntime;

use super::super::template_node_conversion::to_host_contract_template_node;
use super::template_node_projection::project_nodes;

fn to_host_contract_animation_editor_pane(
    data: &AnimationEditorPaneViewData,
) -> host_contract::AnimationEditorPaneData {
    host_contract::AnimationEditorPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node),
        mode: data.mode.clone(),
        asset_path: data.asset_path.clone(),
        status: data.status.clone(),
        selection: data.selection.clone(),
        current_frame: data.current_frame,
        timeline_start_frame: data.timeline_start_frame,
        timeline_end_frame: data.timeline_end_frame,
        playback_label: data.playback_label.clone(),
        track_items: data.track_items.clone(),
        parameter_items: data.parameter_items.clone(),
        node_items: data.node_items.clone(),
        state_items: data.state_items.clone(),
        transition_items: data.transition_items.clone(),
    }
}

pub(crate) fn to_host_contract_animation_editor_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::AnimationEditorPaneData {
    animation_template_projection(data, content_size, None)
        .unwrap_or_else(|| to_host_contract_animation_editor_pane(&data.native_body.animation))
}

pub(crate) fn to_host_contract_animation_editor_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::AnimationEditorPaneData {
    animation_template_projection(data, content_size, Some(runtime))
        .unwrap_or_else(|| to_host_contract_animation_editor_pane(&data.native_body.animation))
}

fn animation_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> Option<host_contract::AnimationEditorPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let nodes = model_rc(super::project_pane_template_nodes_with_runtime(
        &presentation.body,
        content_size,
        runtime,
    )?);

    match &presentation.body.payload {
        PanePayload::AnimationSequenceV1(payload) => Some(host_contract::AnimationEditorPaneData {
            nodes,
            mode: payload.mode.clone().into(),
            asset_path: payload.asset_path.clone().into(),
            status: payload.status.clone().into(),
            selection: payload.selection.clone().into(),
            current_frame: i32::try_from(payload.current_frame).unwrap_or(i32::MAX),
            timeline_start_frame: i32::try_from(payload.timeline_start_frame).unwrap_or(i32::MAX),
            timeline_end_frame: i32::try_from(payload.timeline_end_frame).unwrap_or(i32::MAX),
            playback_label: payload.playback_label.clone().into(),
            track_items: shared_string_list(&payload.track_items),
            parameter_items: ModelRc::default(),
            node_items: ModelRc::default(),
            state_items: ModelRc::default(),
            transition_items: ModelRc::default(),
        }),
        PanePayload::AnimationGraphV1(payload) => Some(host_contract::AnimationEditorPaneData {
            nodes,
            mode: payload.mode.clone().into(),
            asset_path: payload.asset_path.clone().into(),
            status: payload.status.clone().into(),
            selection: payload.selection.clone().into(),
            current_frame: 0,
            timeline_start_frame: 0,
            timeline_end_frame: 0,
            playback_label: String::new().into(),
            track_items: ModelRc::default(),
            parameter_items: shared_string_list(&payload.parameter_items),
            node_items: shared_string_list(&payload.node_items),
            state_items: shared_string_list(&payload.state_items),
            transition_items: shared_string_list(&payload.transition_items),
        }),
        _ => None,
    }
}

fn shared_string_list(items: &[String]) -> ModelRc<SharedString> {
    let mut output = Vec::with_capacity(items.len());
    for item in items {
        output.push(SharedString::from(item.clone()));
    }
    model_rc(output)
}

#[cfg(test)]
mod optimization_batch_20260830cb_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const ITEMS_PER_SAMPLE: usize = 256;

    #[test]
    fn animation_shared_string_list_reserves_input_capacity() {
        let source = include_str!("animation_projection.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(items.len())"));
        assert!(implementation.contains("for item in items"));
        assert!(!implementation.contains("items.iter().cloned().map(SharedString::from).collect()"));
    }

    #[test]
    fn animation_shared_string_list_keeps_item_order() {
        let source = include_str!("animation_projection.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let loop_start = implementation.find("for item in items").expect("item loop");
        let push = implementation
            .find("output.push(SharedString::from(item.clone()))")
            .expect("item push");
        assert!(loop_start < push);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cb_editor_animation_shared_string_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR326_ANIMATION_SHARED_STRING_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items_per_sample={ITEMS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut output = if optimized {
                Vec::with_capacity(ITEMS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..ITEMS_PER_SAMPLE {
                output.push(index);
            }
            checksum ^= output.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
#[path = "animation_projection/direct_list_tests.rs"]
mod direct_list_tests;
