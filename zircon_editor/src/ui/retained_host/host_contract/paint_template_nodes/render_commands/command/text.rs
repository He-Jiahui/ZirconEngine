use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_text::HostTextLayoutPolicy;
use super::{kind::HostPaintCommandKind, model::HostPaintCommand};

impl HostPaintCommand {
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
        Self::text_with_layout_policy(
            frame,
            clip_frame,
            z_index,
            text,
            foreground_color,
            font_size,
            line_height,
            text_style,
            HostTextLayoutPolicy::SingleLineEllipsis,
            opacity,
        )
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn wrapped_text(
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
        Self::text_with_layout_policy(
            frame,
            clip_frame,
            z_index,
            text,
            foreground_color,
            font_size,
            line_height,
            text_style,
            HostTextLayoutPolicy::WordWrap,
            opacity,
        )
    }

    fn text_with_layout_policy(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        text: String,
        foreground_color: [u8; 4],
        font_size: f32,
        line_height: f32,
        text_style: UiTextRunPaintStyle,
        text_layout_policy: HostTextLayoutPolicy,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Text,
            frame,
            clip_frame,
            z_index,
            source_render_command_ref: None,
            background_color: None,
            foreground_color: Some(foreground_color),
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            text: Some(text),
            font_size,
            line_height,
            text_style,
            text_layout_policy,
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }
}
