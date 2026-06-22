use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::{draw_border_clipped, draw_text_bars_clipped};
use super::super::{ACCENT, MUTED_TEXT, SEPARATOR};
use super::geometry::scrolled_menu_frame;

pub(in crate::ui::retained_host::host_contract) fn draw_menu_bar_labels(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: scene
            .layout
            .status_bar_frame
            .width
            .max(scene.layout.center_band_frame.width),
        height: scene.menu_chrome.top_bar_height_px.max(0.0),
    };
    for row in 0..scene.menu_chrome.menu_frames.row_count() {
        let Some(menu_frame) = scene.menu_chrome.menu_frames.row_data(row) else {
            continue;
        };
        let Some(menu) = scene.menu_chrome.menus.row_data(row) else {
            continue;
        };
        let color = if presentation.menu_state.open_menu_index == row as i32 {
            ACCENT
        } else {
            MUTED_TEXT
        };
        let frame_rect = scrolled_menu_frame(&menu_frame.frame, presentation);
        draw_text_bars_clipped(
            frame,
            frame_rect.x + 6.0,
            frame_rect.y + 5.0,
            menu.label.as_str(),
            Some(&clip),
            color,
        );
        draw_border_clipped(frame, frame_rect, Some(&clip), SEPARATOR);
    }
}
