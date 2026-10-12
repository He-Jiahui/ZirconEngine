use super::super::chrome_command_stream::{
    build_chrome_command_stream, paint_chrome_command_stream_to_frame,
};
use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;

pub(in crate::ui::retained_host::host_contract) fn paint_host_presentation_snapshot(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> HostRgbaFrame {
    let stream = build_chrome_command_stream(presentation, (width, height), None, true);
    paint_chrome_command_stream_to_frame(width, height, &stream)
}
