use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostPaintAtlasImage;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::visual_assets::HostPaintImagePixels;
use super::{kind::HostPaintCommandKind, model::HostPaintCommand};

const FALLBACK_IMAGE_BORDER: [u8; 4] = PALETTE.focus_ring;

impl HostPaintCommand {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        image_key: String,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Image,
            frame,
            clip_frame,
            z_index,
            background_color: None,
            foreground_color: None,
            border_color: Some(FALLBACK_IMAGE_BORDER),
            border_width: 1.0,
            corner_radius: 0.0,
            text: None,
            font_size: 12.0,
            line_height: 14.0,
            text_style: Default::default(),
            image_key: Some(image_key),
            image_pixels: None,
            opacity,
        }
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image_pixels(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        resource_key: String,
        image_width: u32,
        image_height: u32,
        rgba: Vec<u8>,
        atlas: Option<HostPaintAtlasImage>,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Image,
            frame,
            clip_frame,
            z_index,
            background_color: None,
            foreground_color: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            text: None,
            font_size: 12.0,
            line_height: 14.0,
            text_style: Default::default(),
            image_key: None,
            image_pixels: Some(HostPaintImagePixels {
                resource_key,
                width: image_width,
                height: image_height,
                rgba,
                atlas,
            }),
            opacity,
        }
    }
}
