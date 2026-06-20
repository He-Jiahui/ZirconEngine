use super::super::*;
use crate::ui::workbench::autolayout::ShellFrame;

pub(crate) fn viewport_size_from_frame(frame: ShellFrame) -> Option<UVec2> {
    let width = frame.width.max(0.0).round() as u32;
    let height = frame.height.max(0.0).round() as u32;
    if width == 0 || height == 0 {
        None
    } else {
        Some(UVec2::new(width, height))
    }
}

pub(crate) fn compute_window_menu_popup_height(
    shell_height: f32,
    button_frame: UiFrame,
    preset_count: usize,
) -> f32 {
    let popup_y = button_frame.y + button_frame.height + 3.0;
    let content_height = 72.0 + preset_count as f32 * 30.0;
    let available_height = (shell_height - popup_y - 12.0).max(72.0);
    content_height.min(available_height)
}

pub(crate) fn shell_region_group_key(region: ShellRegionId) -> &'static str {
    match region {
        ShellRegionId::Left => "left",
        ShellRegionId::Right => "right",
        ShellRegionId::Bottom => "bottom",
        ShellRegionId::Document => "document",
    }
}
