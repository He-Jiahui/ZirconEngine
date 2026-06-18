use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostPaintAtlasImage;
use super::super::super::paint_theme::PALETTE;
use super::super::visual_assets::HostPaintImagePixels;

const FALLBACK_IMAGE_BORDER: [u8; 4] = PALETTE.focus_ring;

#[derive(Clone, Copy)]
pub(super) enum HostPaintCommandKind {
    Group,
    Quad,
    Text,
    Image,
}

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct HostPaintCommand {
    pub(super) kind: HostPaintCommandKind,
    pub(super) frame: FrameRect,
    pub(super) clip_frame: Option<FrameRect>,
    pub(super) z_index: i32,
    pub(super) background_color: Option<[u8; 4]>,
    pub(super) foreground_color: Option<[u8; 4]>,
    pub(super) border_color: Option<[u8; 4]>,
    pub(super) border_width: f32,
    pub(super) corner_radius: f32,
    pub(super) text: Option<String>,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) text_style: UiTextRunPaintStyle,
    pub(super) image_key: Option<String>,
    pub(super) image_pixels: Option<HostPaintImagePixels>,
    pub(super) opacity: f32,
}

impl HostPaintCommand {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn quad(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        background_color: Option<[u8; 4]>,
        border_color: Option<[u8; 4]>,
        border_width: f32,
        corner_radius: f32,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Quad,
            frame,
            clip_frame,
            z_index,
            background_color,
            foreground_color: None,
            border_color,
            border_width,
            corner_radius,
            text: None,
            font_size: 12.0,
            line_height: 14.0,
            text_style: UiTextRunPaintStyle::default(),
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        text: String,
        foreground_color: [u8; 4],
        font_size: f32,
        line_height: f32,
        text_style: UiTextRunPaintStyle,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Text,
            frame,
            clip_frame,
            z_index,
            background_color: None,
            foreground_color: Some(foreground_color),
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            text: Some(text),
            font_size,
            line_height,
            text_style,
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn group(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Group,
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
            text_style: UiTextRunPaintStyle::default(),
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }

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
            text_style: UiTextRunPaintStyle::default(),
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
            text_style: UiTextRunPaintStyle::default(),
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
