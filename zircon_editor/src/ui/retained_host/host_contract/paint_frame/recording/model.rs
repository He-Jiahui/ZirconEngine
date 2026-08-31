use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{UiRenderFrameCommandRef, UiTextRunPaintStyle};

use super::super::super::data::FrameRect;
use super::source_table::{HostRenderSourceKey, HostRenderSourceTable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) struct HostRenderCommandSource {
    pub(in crate::ui::retained_host::host_contract) surface_key: HostRenderSourceKey,
    pub(in crate::ui::retained_host::host_contract) command_ref: UiRenderFrameCommandRef,
    pub(in crate::ui::retained_host::host_contract) fragment_index: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostPaintImageUvRect {
    pub(in crate::ui::retained_host::host_contract) min: [f32; 2],
    pub(in crate::ui::retained_host::host_contract) max: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostPaintAtlasImage {
    pub(in crate::ui::retained_host::host_contract) resource_key: String,
    pub(in crate::ui::retained_host::host_contract) resource_generation: u64,
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
    /// Legacy test and explicit CPU fallback payload; production atlas handles leave this empty.
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
        rgba: Option<Arc<[u8]>>,
        atlas: Option<HostPaintAtlasImage>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostRecordedPaintCommand {
    pub frame: FrameRect,
    pub clip_frame: Option<FrameRect>,
    pub z_index: i32,
    pub source: Option<HostRenderCommandSource>,
    pub kind: HostRecordedPaintKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostRecordedFrame {
    pub commands: Vec<HostRecordedPaintCommand>,
    pub render_sources: HostRenderSourceTable,
}
