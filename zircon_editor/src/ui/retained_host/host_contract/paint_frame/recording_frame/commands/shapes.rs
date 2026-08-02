use super::super::super::super::data::FrameRect;
use super::super::super::HostRgbaFrame;
use super::super::super::recording::HostRecordedPaintKind;
use super::record::record_command;

impl HostRgbaFrame {
    pub(in crate::ui::retained_host::host_contract) fn record_quad(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        color: [u8; 4],
        corner_radius: f32,
    ) {
        record_command(
            self,
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
        record_command(
            self,
            frame,
            clip_frame,
            HostRecordedPaintKind::Border {
                color,
                width,
                corner_radius,
            },
        );
    }
}
