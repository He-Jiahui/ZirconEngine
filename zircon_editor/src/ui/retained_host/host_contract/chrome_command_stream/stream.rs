use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use crate::ui::retained_host::host_contract::data::FrameRect;

use super::command::{ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeCommandStream {
    surface_size: (u32, u32),
    damage: Option<FrameRect>,
    full_rebuild: bool,
    pub(super) commands: Vec<ChromeCommand>,
}

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn full_rebuild(
        surface_size: (u32, u32),
    ) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: None,
            full_rebuild: true,
            commands: Vec::new(),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn patch(
        surface_size: (u32, u32),
        damage: FrameRect,
    ) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: Some(damage),
            full_rebuild: false,
            commands: Vec::new(),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn is_full_rebuild(&self) -> bool {
        self.full_rebuild
    }

    pub(in crate::ui::retained_host::host_contract) fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub(in crate::ui::retained_host::host_contract) fn damage(&self) -> Option<&FrameRect> {
        self.damage.as_ref()
    }

    pub(in crate::ui::retained_host::host_contract) fn commands(&self) -> &[ChromeCommand] {
        &self.commands
    }

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

    pub(super) fn extend_commands(&mut self, commands: impl IntoIterator<Item = ChromeCommand>) {
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

pub(super) fn clamp_surface_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}
