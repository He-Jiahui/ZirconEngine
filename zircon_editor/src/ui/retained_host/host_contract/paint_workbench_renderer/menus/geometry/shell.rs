use super::super::super::super::data::HostWindowPresentationData;

pub(super) fn menu_shell_width(presentation: &HostWindowPresentationData) -> f32 {
    presentation
        .host_layout
        .status_bar_frame
        .width
        .max(presentation.host_scene_data.layout.status_bar_frame.width)
        .max(presentation.host_scene_data.layout.center_band_frame.width)
        .max(1.0)
}

pub(super) fn menu_shell_height(presentation: &HostWindowPresentationData) -> f32 {
    presentation
        .host_layout
        .status_bar_frame
        .y
        .max(presentation.host_scene_data.layout.status_bar_frame.y)
        .max(1.0)
}
