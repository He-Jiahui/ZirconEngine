mod dispatchable;
mod source;

use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
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
        if !is_dispatchable_template_node(&node) {
            continue;
        }
        let frame = translated_template_frame(&node.frame, body.x, body.y);
        let clip = node
            .has_clip_frame
            .then(|| translated_template_frame(&node.clip_frame, body.x, body.y).into());
        let effective_frame = if let Some(clip_frame) = clip.as_ref() {
            let Some(frame) = intersect_profile_frame(&frame, clip_frame) else {
                continue;
            };
            frame
        } else {
            frame.clone().into()
        };
        if !is_visible_profile_frame(&effective_frame) {
            continue;
        }
        push_named_profile_frame(
            out,
            format!("template.{surface}.{}", node.control_id).as_str(),
            "template_control",
            surface,
            effective_frame,
            clip,
        );
    }
}
