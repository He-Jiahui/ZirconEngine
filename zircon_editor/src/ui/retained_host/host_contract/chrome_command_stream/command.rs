use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use crate::ui::retained_host::host_contract::data::FrameRect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum ChromeCommandLayer {
    Static,
    Dynamic,
    Text,
    Viewport,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum ChromeCommandKind {
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
        size: f32,
        line_height: f32,
        style: UiTextRunPaintStyle,
    },
    Image {
        payload: ChromeImagePayload,
    },
    Clip,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeImagePayload {
    pub(in crate::ui::retained_host::host_contract) resource_key: String,
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
    pub(in crate::ui::retained_host::host_contract) upload_bytes: u64,
    pub(in crate::ui::retained_host::host_contract) rgba: Option<Vec<u8>>,
    pub(in crate::ui::retained_host::host_contract) atlas_uv: Option<ChromeImageUvRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeImageUvRect {
    pub(in crate::ui::retained_host::host_contract) min: [f32; 2],
    pub(in crate::ui::retained_host::host_contract) max: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeCommand {
    pub(in crate::ui::retained_host::host_contract) layer: ChromeCommandLayer,
    pub(in crate::ui::retained_host::host_contract) z_index: i32,
    pub(in crate::ui::retained_host::host_contract) frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract) clip: Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract) kind: ChromeCommandKind,
}
