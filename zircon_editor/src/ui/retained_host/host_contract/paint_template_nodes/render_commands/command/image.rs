use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostPaintAtlasImage;
use super::super::super::super::paint_text::HostTextLayoutPolicy;
use super::super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};
use super::super::super::visual_assets::HostPaintImagePixels;
use super::{kind::HostPaintCommandKind, model::HostPaintCommand};

fn fallback_image_border_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.border
}

fn fallback_image_frame_metrics_from_host(metrics: HostControlMetrics) -> (f32, f32) {
    (metrics.border_width, metrics.radius_control)
}

impl HostPaintCommand {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        image_key: String,
        opacity: f32,
    ) -> Self {
        let metrics = current_host_metrics();
        let (border_width, corner_radius) = fallback_image_frame_metrics_from_host(metrics);
        let (font_size, line_height) = Self::fallback_text_metrics();
        Self {
            kind: HostPaintCommandKind::Image,
            frame,
            clip_frame,
            z_index,
            background_color: None,
            foreground_color: None,
            border_color: Some(fallback_image_border_from_host(current_host_palette())),
            border_width,
            corner_radius,
            text: None,
            font_size,
            line_height,
            text_style: Default::default(),
            text_layout_policy: HostTextLayoutPolicy::SingleLineEllipsis,
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
        let (font_size, line_height) = Self::fallback_text_metrics();
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
            font_size,
            line_height,
            text_style: Default::default(),
            text_layout_policy: HostTextLayoutPolicy::SingleLineEllipsis,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

    #[test]
    fn render_command_image_border_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.border = [10, 11, 12, 255];
        palette.focus_ring = [13, 14, 15, 255];

        assert_eq!(fallback_image_border_from_host(palette), [10, 11, 12, 255]);
    }

    #[test]
    fn render_command_image_frame_metrics_project_from_host_metrics() {
        assert_eq!(
            fallback_image_frame_metrics_from_host(METRICS),
            (METRICS.border_width, METRICS.radius_control)
        );
    }

    #[test]
    fn render_command_image_fallback_text_metrics_project_from_host_metrics() {
        assert_eq!(
            HostPaintCommand::fallback_text_metrics_from_host(METRICS),
            (METRICS.font_body, METRICS.line_height(METRICS.font_body))
        );
    }
}
