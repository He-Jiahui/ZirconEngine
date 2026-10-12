#[path = "menus/bar.rs"]
mod bar;
#[path = "menus/geometry.rs"]
mod geometry;
#[path = "menus/popup.rs"]
mod popup;
#[path = "menus/rows.rs"]
mod rows;

use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;

pub(in crate::ui::retained_host::host_contract) fn draw_menu_bar_labels(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    bar::draw_menu_bar_labels(frame, presentation);
}

pub(in crate::ui::retained_host::host_contract) fn draw_open_menu_popup(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    popup::draw_open_menu_popup(frame, presentation);
}
