mod bar;
mod geometry;
mod popup;
mod rows;

use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;

pub(super) fn draw_menu_bar_labels(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    bar::draw_menu_bar_labels(frame, presentation);
}

pub(super) fn draw_open_menu_popup(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    popup::draw_open_menu_popup(frame, presentation);
}
