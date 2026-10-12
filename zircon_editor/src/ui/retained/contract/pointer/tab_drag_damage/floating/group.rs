use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::union::visible_frame;

pub(in super::super) fn floating_group_frame(
    presentation: &HostWindowPresentationData,
    group: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    for window in scene.floating_layer.floating_windows.iter() {
        if group == window.target_group.as_str()
            || group == window.left_edge_target_group.as_str()
            || group == window.right_edge_target_group.as_str()
            || group == window.top_edge_target_group.as_str()
            || group == window.bottom_edge_target_group.as_str()
        {
            return visible_frame(&window.frame).then_some(window.frame.clone());
        }
    }
    None
}
