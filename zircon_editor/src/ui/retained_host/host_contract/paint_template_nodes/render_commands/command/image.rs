use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostPaintAtlasImage;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::visual_assets::HostPaintImagePixels;
use super::{kind::HostPaintCommandKind, model::HostPaintCommand};

fn fallback_image_border_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.focus_ring
}

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
            border_color: Some(fallback_image_border_from_host(current_host_palette())),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn render_command_image_border_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.focus_ring = [10, 11, 12, 255];

        assert_eq!(fallback_image_border_from_host(palette), [10, 11, 12, 255]);
    }
}
