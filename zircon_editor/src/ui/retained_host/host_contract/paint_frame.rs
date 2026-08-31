mod frame;
mod geometry;
mod pixels;
mod recording;
mod recording_frame;

pub(in crate::ui::retained_host::host_contract) use frame::HostRgbaFrame;
pub(in crate::ui::retained_host::host_contract) use recording::{
    HostPaintAtlasImage, HostPaintImageUvRect, HostRecordedFrame, HostRecordedPaintCommand,
    HostRecordedPaintKind, HostRenderCommandSource, HostRenderSourceKey, HostRenderSourceTable,
};

#[cfg(test)]
mod tests;
