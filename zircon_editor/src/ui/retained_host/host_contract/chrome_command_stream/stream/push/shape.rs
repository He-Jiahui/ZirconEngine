use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::command::{ChromeCommandKind, ChromeCommandLayer};
use super::super::model::ChromeCommandStream;

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn push_quad(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        color: [u8; 4],
        corner_radius: f32,
    ) {
        self.push_command(
            layer,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Quad {
                color,
                corner_radius,
            },
        );
    }

    pub(in crate::ui::retained_host::host_contract) fn push_border(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    ) {
        self.push_command(
            layer,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Border {
                color,
                width,
                corner_radius,
            },
        );
    }
}
