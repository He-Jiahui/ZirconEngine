use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::command::{
    ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload,
};
use super::geometry::visible_frame;
use super::model::ChromeCommandStream;

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

    pub(in crate::ui::retained_host::host_contract) fn extend_commands(
        &mut self,
        commands: impl IntoIterator<Item = ChromeCommand>,
    ) {
        self.commands.extend(commands);
    }

    fn push_command(
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
        self.commands.push(ChromeCommand {
            layer,
            z_index,
            frame,
            clip,
            kind,
        });
    }
}
