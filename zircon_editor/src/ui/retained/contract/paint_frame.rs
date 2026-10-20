#[path = "paint_frame/frame.rs"]
mod frame;
#[path = "paint_frame/geometry.rs"]
mod geometry;
#[path = "paint_frame/pixels.rs"]
mod pixels;
#[path = "paint_frame/recording.rs"]
mod recording;
#[path = "paint_frame/recording_frame.rs"]
mod recording_frame;

pub(in crate::ui::retained_host::host_contract) use frame::HostRgbaFrame;
pub(in crate::ui::retained_host::host_contract) use recording::{
    HostPaintAtlasImage, HostPaintImageUvRect, HostPaintRecording, HostRecordedFrame,
    HostRecordedPaintCommand, HostRecordedPaintKind, HostRenderCommandSource, HostRenderSourceKey,
    HostRenderSourceTable,
};

#[cfg(test)]
#[path = "paint_frame/tests/cases.rs"]
mod tests;
