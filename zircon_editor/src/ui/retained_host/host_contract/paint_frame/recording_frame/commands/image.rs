use super::super::super::super::data::FrameRect;
use super::super::super::recording::{HostPaintAtlasImage, HostRecordedPaintKind};
use super::super::super::HostRgbaFrame;
use super::record::record_command;

impl HostRgbaFrame {
    pub(in crate::ui::retained_host::host_contract) fn record_image(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        resource_key: impl Into<String>,
        width: u32,
        height: u32,
        rgba: Option<Vec<u8>>,
        atlas: Option<HostPaintAtlasImage>,
    ) {
        record_command(
            self,
            frame,
            clip_frame,
            HostRecordedPaintKind::Image {
                resource_key: resource_key.into(),
                width,
                height,
                rgba,
                atlas,
            },
        );
    }
}
