use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::data::FrameRect;
use super::recording::{
    HostPaintAtlasImage, HostPaintRecording, HostRecordedPaintCommand, HostRecordedPaintKind,
};
use super::HostRgbaFrame;

impl HostRgbaFrame {
    pub(in crate::ui::retained_host::host_contract) fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    pub(in crate::ui::retained_host::host_contract) fn record_only(&self) -> bool {
        self.recording
            .as_ref()
            .is_some_and(HostPaintRecording::is_record_only)
    }

    pub(in crate::ui::retained_host::host_contract) fn record_quad(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        color: [u8; 4],
        corner_radius: f32,
    ) {
        self.record_command(
            frame,
            clip_frame,
            HostRecordedPaintKind::Quad {
                color,
                corner_radius,
            },
        );
    }

    pub(in crate::ui::retained_host::host_contract) fn record_border(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    ) {
        self.record_command(
            frame,
            clip_frame,
            HostRecordedPaintKind::Border {
                color,
                width,
                corner_radius,
            },
        );
    }

    pub(in crate::ui::retained_host::host_contract) fn record_text(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        text: impl Into<String>,
        color: [u8; 4],
        font_size: f32,
        line_height: f32,
        style: UiTextRunPaintStyle,
    ) {
        self.record_command(
            frame,
            clip_frame,
            HostRecordedPaintKind::Text {
                text: text.into(),
                color,
                font_size,
                line_height,
                style,
            },
        );
    }

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
        self.record_command(
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

    pub(in crate::ui::retained_host::host_contract) fn into_recorded_commands(
        self,
    ) -> Vec<HostRecordedPaintCommand> {
        self.recording
            .map(HostPaintRecording::into_commands)
            .unwrap_or_default()
    }

    fn record_command(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        kind: HostRecordedPaintKind,
    ) {
        if let Some(recording) = self.recording.as_mut() {
            recording.record_command(frame, clip_frame, kind);
        }
    }
}
