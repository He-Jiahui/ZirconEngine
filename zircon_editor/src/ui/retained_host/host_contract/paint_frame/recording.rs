use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::data::FrameRect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostPaintImageUvRect {
    pub(in crate::ui::retained_host::host_contract) min: [f32; 2],
    pub(in crate::ui::retained_host::host_contract) max: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostPaintAtlasImage {
    pub(in crate::ui::retained_host::host_contract) resource_key: String,
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
    pub(in crate::ui::retained_host::host_contract) rgba: Option<Vec<u8>>,
    pub(in crate::ui::retained_host::host_contract) uv: HostPaintImageUvRect,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum HostRecordedPaintKind {
    Quad {
        color: [u8; 4],
        corner_radius: f32,
    },
    Border {
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    },
    Text {
        text: String,
        color: [u8; 4],
        font_size: f32,
        line_height: f32,
        style: UiTextRunPaintStyle,
    },
    Image {
        resource_key: String,
        width: u32,
        height: u32,
        rgba: Option<Vec<u8>>,
        atlas: Option<HostPaintAtlasImage>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostRecordedPaintCommand {
    pub frame: FrameRect,
    pub clip_frame: Option<FrameRect>,
    pub z_index: i32,
    pub kind: HostRecordedPaintKind,
}

#[derive(Clone, Debug, Default)]
pub(in crate::ui::retained_host::host_contract) struct HostPaintRecording {
    commands: Vec<HostRecordedPaintCommand>,
    next_z_index: i32,
    record_only: bool,
}

impl HostPaintRecording {
    pub(in crate::ui::retained_host::host_contract) fn record_only() -> Self {
        Self {
            record_only: true,
            ..Self::default()
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn is_record_only(&self) -> bool {
        self.record_only
    }

    pub(in crate::ui::retained_host::host_contract) fn into_commands(
        self,
    ) -> Vec<HostRecordedPaintCommand> {
        self.commands
    }

    pub(in crate::ui::retained_host::host_contract) fn record_command(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        kind: HostRecordedPaintKind,
    ) {
        if !super::geometry::visible_frame(&frame) {
            return;
        }
        let z_index = self.next_z_index;
        self.next_z_index = self.next_z_index.saturating_add(1);
        self.commands.push(HostRecordedPaintCommand {
            frame,
            clip_frame,
            z_index,
            kind,
        });
    }
}
