use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::union::visible_frame;

pub(super) fn local_group_frame(
    presentation: &HostWindowPresentationData,
    group: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let frame = match group {
        "left" => scene.left_dock.region_frame.clone(),
        "document" => scene.document_dock.region_frame.clone(),
        "right" => scene.right_dock.region_frame.clone(),
        "bottom" => scene.bottom_dock.region_frame.clone(),
        _ => return None,
    };
    visible_frame(&frame).then_some(frame)
}

pub(super) fn document_edge_group(group: &str) -> bool {
    matches!(
        group,
        "document-left" | "document-right" | "document-top" | "document-bottom"
    )
}
