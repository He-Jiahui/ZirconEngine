use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::command::{ChromeCommand, ChromeCommandKind, ChromeCommandLayer};
use super::super::geometry::visible_frame;
use super::super::model::ChromeCommandStream;

impl ChromeCommandStream {
    pub(super) fn push_command(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        kind: ChromeCommandKind,
    ) {
        if !visible_frame(&frame) {
            return;
        }
        self.image_resources_compacted = false;
        self.commands.push(ChromeCommand {
            layer,
            z_index,
            frame,
            clip,
            source: None,
            kind,
        });
    }
}
