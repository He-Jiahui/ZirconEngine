use super::super::super::super::data::FrameRect;
use super::{kind::HostPaintCommandKind, model::HostPaintCommand};

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
        let (font_size, line_height) = Self::fallback_text_metrics();
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
            font_size,
            line_height,
            text_style: Default::default(),
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }
}
