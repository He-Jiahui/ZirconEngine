use super::super::super::super::data::HostWindowPresentationData;

// 菜单绘制边界取宿主布局与场景投影中较大的壳尺寸，供根菜单和子菜单约束。
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
