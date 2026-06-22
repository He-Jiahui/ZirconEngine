use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::dimensions::shell_content_width;

pub(in crate::ui::retained_host::host_contract) fn popup_blocking_frame(
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    let top_bar_height = presentation
        .host_scene_data
        .menu_chrome
        .top_bar_height_px
        .max(0.0);
    let status_bar_top = presentation
        .host_layout
        .status_bar_frame
        .y
        .max(presentation.host_scene_data.layout.status_bar_frame.y);

    FrameRect {
        x: 0.0,
        y: top_bar_height,
        width: shell_content_width(presentation),
        height: (status_bar_top - top_bar_height).max(0.0),
    }
}
