use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::command::{ChromeCommandKind, ChromeCommandLayer};
use super::super::model::ChromeCommandStream;

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn push_clip(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
    ) {
        self.push_command(
            layer,
            z_index,
            frame.clone(),
            Some(frame),
            ChromeCommandKind::Clip,
        );
    }
}
