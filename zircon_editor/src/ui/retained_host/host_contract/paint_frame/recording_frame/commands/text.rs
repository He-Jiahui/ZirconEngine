use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::data::FrameRect;
use super::super::super::HostRgbaFrame;
use super::super::super::recording::HostRecordedPaintKind;
use super::record::record_command;

impl HostRgbaFrame {
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
        record_command(
            self,
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
}
