use super::super::super::super::data::FrameRect;
use super::{kind::HostPaintCommandKind, model::HostPaintCommand};

impl HostPaintCommand {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn group(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        opacity: f32,
    ) -> Self {
        let (font_size, line_height) = Self::fallback_text_metrics();
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
            font_size,
            line_height,
            text_style: Default::default(),
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }
}
