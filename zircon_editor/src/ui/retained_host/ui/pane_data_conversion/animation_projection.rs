use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    AnimationEditorPaneViewData, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::template_runtime::EditorUiHostRuntime;

use super::super::template_node_conversion::to_host_contract_template_node_owned;
use super::template_node_projection::project_nodes;

fn to_host_contract_animation_editor_pane(
    data: AnimationEditorPaneViewData,
) -> host_contract::AnimationEditorPaneData {
    host_contract::AnimationEditorPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node_owned),
        mode: data.mode,
        asset_path: data.asset_path,
        status: data.status,
        selection: data.selection,
        current_frame: data.current_frame,
        timeline_start_frame: data.timeline_start_frame,
        timeline_end_frame: data.timeline_end_frame,
        playback_label: data.playback_label,
        track_items: data.track_items,
        parameter_items: data.parameter_items,
        node_items: data.node_items,
        state_items: data.state_items,
        transition_items: data.transition_items,
    }
}

pub(crate) fn to_host_contract_animation_editor_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::AnimationEditorPaneData {
    animation_template_projection(data, content_size, None).unwrap_or_else(|| {
        to_host_contract_animation_editor_pane(data.native_body.animation.clone())
    })
}

pub(crate) fn to_host_contract_animation_editor_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::AnimationEditorPaneData {
    animation_template_projection(data, content_size, Some(runtime)).unwrap_or_else(|| {
        to_host_contract_animation_editor_pane(data.native_body.animation.clone())
    })
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
            track_items: shared_string_list(payload.track_items.clone()),
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
            parameter_items: shared_string_list(payload.parameter_items.clone()),
            node_items: shared_string_list(payload.node_items.clone()),
            state_items: shared_string_list(payload.state_items.clone()),
            transition_items: shared_string_list(payload.transition_items.clone()),
        }),
        _ => None,
    }
}

fn shared_string_list(items: Vec<String>) -> ModelRc<SharedString> {
    model_rc(items.into_iter().map(SharedString::from).collect())
}
