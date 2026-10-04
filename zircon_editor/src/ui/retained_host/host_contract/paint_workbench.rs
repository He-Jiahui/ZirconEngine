mod commands;

#[cfg(test)]
#[path = "paint_workbench/tests/test_frame.rs"]
mod test_frame;

use super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract) use commands::draw_workbench_presentation_commands;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use test_frame::{
    paint_host_frame, repaint_host_frame_region,
};

pub(crate) fn paint_product_presentation_for_evidence(
    width: u32,
    height: u32,
    presentation: &super::data::HostWindowPresentationData,
) -> Vec<u8> {
    let mut frame =
        super::paint_frame::HostRgbaFrame::filled(width, height, PALETTE.shell_background);
    draw_workbench_presentation_commands(&mut frame, presentation);
    frame.into_bytes()
}
