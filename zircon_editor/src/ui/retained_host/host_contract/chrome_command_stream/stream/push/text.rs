use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::command::{ChromeCommandKind, ChromeCommandLayer};
use super::super::model::ChromeCommandStream;

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn push_text(
        &mut self,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        text: impl Into<String>,
        color: [u8; 4],
        size: f32,
    ) {
        self.push_command(
            ChromeCommandLayer::Text,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Text {
                text: text.into(),
                color,
                size,
                line_height: size.max(1.0) * 1.2,
                style: UiTextRunPaintStyle::default(),
            },
        );
    }
}
