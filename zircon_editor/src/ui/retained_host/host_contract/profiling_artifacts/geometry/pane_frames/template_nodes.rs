mod dispatchable;
mod source;

use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::profiling_artifacts::UiProfileNamedFrame;

use self::dispatchable::is_dispatchable_template_node;
use self::source::pane_template_nodes;

use super::super::frame_math::{
    intersect_profile_frame, is_visible_profile_frame, push_named_profile_frame,
    translated_template_frame,
};

pub(super) fn collect_template_node_controls(
    surface: &str,
    pane: &PaneData,
    body: &FrameRect,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    let Some(nodes) = pane_template_nodes(pane) else {
        return;
    };
    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        collect_template_node_control(surface, &node, body, None, out);
    }
}

pub(in crate::ui::retained_host::host_contract) fn collect_template_node_control(
    surface: &str,
    node: &TemplatePaneNodeData,
    body: &FrameRect,
    pane_clip: Option<&FrameRect>,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    if !is_dispatchable_template_node(node) {
        return;
    }
    let frame = translated_template_frame(&node.frame, body.x, body.y);
    let mut clip = node
        .has_clip_frame
        .then(|| translated_template_frame(&node.clip_frame, body.x, body.y).into());
    let mut effective_frame = if let Some(clip_frame) = clip.as_ref() {
        let Some(frame) = intersect_profile_frame(&frame, clip_frame) else {
            return;
        };
        frame
    } else {
        frame.clone().into()
    };
    if let Some(pane_clip) = pane_clip {
        let pane_clip = pane_clip.clone().into();
        clip = match clip {
            Some(node_clip) => intersect_profile_frame(&profile_frame_rect(&node_clip), &pane_clip),
            None => Some(pane_clip),
        };
        let Some(pane_clip) = clip.as_ref() else {
            return;
        };
        let Some(frame) = intersect_profile_frame(&profile_frame_rect(&effective_frame), pane_clip)
        else {
            return;
        };
        effective_frame = frame;
    }
    if !is_visible_profile_frame(&effective_frame) {
        return;
    }
    push_named_profile_frame(
        out,
        format!("template.{surface}.{}", node.control_id),
        "template_control",
        surface,
        effective_frame,
        clip,
    );
}

fn profile_frame_rect(
    frame: &crate::ui::retained_host::host_contract::profiling_artifacts::UiProfileFrame,
) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
