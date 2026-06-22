use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::command::{ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload};
use super::super::model::ChromeCommandStream;

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn push_image(
        &mut self,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        payload: ChromeImagePayload,
    ) {
        self.push_command(
            ChromeCommandLayer::Viewport,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Image { payload },
        );
    }
}
