use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;
use super::visual_assets::load_existing_icon_asset_pixels_for_size;

const VISUAL_SURFACE_INSET_RATIO: f32 = 0.12;
const VISUAL_SURFACE_MIN_INSET: f32 = 5.0;
const VISUAL_SURFACE_MAX_INSET: f32 = 8.0;
const VISUAL_SURFACE_RADIUS: f32 = 4.0;
const THUMBNAIL_ICON_NAME: &str = "image";
const THUMBNAIL_ICON_EDGE: u32 = 20;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_asset_placeholder_visual_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !is_asset_thumbnail_visual(node) {
        return;
    }

    let Some(inner_rect) = thumbnail_surface_rect(rect) else {
        return;
    };

    commands.push(HostPaintCommand::quad(
        inner_rect.clone(),
        Some(clip.clone()),
        order,
        Some(thumbnail_well_color(node)),
        None,
        0.0,
        VISUAL_SURFACE_RADIUS,
        opacity,
    ));

    push_thumbnail_icon_command(commands, &inner_rect, clip, order + 1, opacity);
}

fn is_asset_thumbnail_visual(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.surface_variant.as_str(),
        "asset-placeholder-visual" | "asset-preview-visual"
    )
}

fn thumbnail_well_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match node.surface_variant.as_str() {
        "asset-preview-visual" => PALETTE.surface_inset,
        _ => PALETTE.surface,
    }
}

fn thumbnail_surface_rect(rect: &FrameRect) -> Option<FrameRect> {
    let shortest_edge = rect.width.min(rect.height);
    if shortest_edge <= VISUAL_SURFACE_MIN_INSET * 2.0 {
        return None;
    }
    let inset = (shortest_edge * VISUAL_SURFACE_INSET_RATIO)
        .clamp(VISUAL_SURFACE_MIN_INSET, VISUAL_SURFACE_MAX_INSET);
    let width = (rect.width - inset * 2.0).max(0.0);
    let height = (rect.height - inset * 2.0).max(0.0);
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some(FrameRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width,
        height,
    })
}

fn push_thumbnail_icon_command(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(edge) = thumbnail_icon_edge(rect) else {
        return;
    };
    let Some(image) = load_existing_icon_asset_pixels_for_size(
        THUMBNAIL_ICON_NAME,
        edge,
        edge,
        Some(PALETTE.text_disabled),
    ) else {
        return;
    };

    commands.push(HostPaintCommand::image_pixels(
        thumbnail_icon_rect(rect, edge),
        Some(clip.clone()),
        order,
        image.resource_key,
        image.width,
        image.height,
        image.rgba,
        image.atlas,
        opacity,
    ));
}

fn thumbnail_icon_edge(rect: &FrameRect) -> Option<u32> {
    let max_edge = rect.width.min(rect.height).floor() as u32;
    let edge = THUMBNAIL_ICON_EDGE.min(max_edge);
    (edge > 0).then_some(edge)
}

fn thumbnail_icon_rect(rect: &FrameRect, edge: u32) -> FrameRect {
    let edge = edge as f32;
    FrameRect {
        x: rect.x + (rect.width - edge) * 0.5,
        y: rect.y + (rect.height - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

#[cfg(test)]
mod tests {
    use super::super::template_node_pipeline::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;

    #[test]
    fn asset_placeholder_visual_uses_single_recessed_well_and_svg_icon() {
        let rect = placeholder_rect();
        let mut commands = Vec::new();

        push_asset_placeholder_visual_commands(
            &mut commands,
            &placeholder_node("asset-placeholder-visual"),
            &rect,
            &rect,
            0,
            1.0,
        );

        let well_commands = commands
            .iter()
            .filter(|command| command.image_pixels.is_none())
            .collect::<Vec<_>>();
        assert_eq!(well_commands.len(), 1);
        assert_eq!(well_commands[0].background_color, Some(PALETTE.surface));
        assert_eq!(well_commands[0].border_color, None);
        assert_eq!(well_commands[0].border_width, 0.0);

        let icon_commands = commands
            .iter()
            .filter(|command| command.image_pixels.is_some())
            .collect::<Vec<_>>();
        assert_eq!(icon_commands.len(), 1);
        let Some(icon) = icon_commands[0].image_pixels.as_ref() else {
            assert!(false, "placeholder icon should use real SVG pixels");
            return;
        };
        assert_eq!((icon.width, icon.height), (20, 20));
        assert!(
            !icon.resource_key.starts_with("missing-icon:"),
            "placeholder icon should resolve through the real shell image icon"
        );
        assert_eq!(icon_commands[0].frame.width, 20.0);
        assert_eq!(icon_commands[0].frame.height, 20.0);
    }

    #[test]
    fn asset_placeholder_visual_paints_recessed_well_without_inner_outline() {
        let bytes = paint_template_nodes_for_test(
            112,
            64,
            model_rc(vec![placeholder_node("asset-placeholder-visual")]),
        );

        assert_eq!(pixel_at(&bytes, 112, 14, 10), PALETTE.surface_inset);
        assert_eq!(pixel_at(&bytes, 112, 49, 14), PALETTE.surface);
    }

    #[test]
    fn asset_placeholder_without_visual_variant_keeps_plain_inset_surface() {
        let bytes = paint_template_nodes_for_test(
            112,
            64,
            model_rc(vec![placeholder_node("asset-placeholder")]),
        );

        assert_eq!(pixel_at(&bytes, 112, 49, 14), PALETTE.surface_inset);
        assert_eq!(pixel_at(&bytes, 112, 49, 29), PALETTE.surface_inset);
        assert_eq!(pixel_at(&bytes, 112, 40, 37), PALETTE.surface_inset);
    }

    #[test]
    fn asset_preview_visual_paints_recessed_well_inside_selected_preview() {
        let bytes = paint_template_nodes_for_test(
            112,
            64,
            model_rc(vec![placeholder_node("asset-preview-visual")]),
        );

        assert_eq!(pixel_at(&bytes, 112, 14, 10), PALETTE.surface);
        assert_eq!(pixel_at(&bytes, 112, 49, 14), PALETTE.surface_inset);
    }

    fn placeholder_node(surface_variant: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "AssetPreviewVisual".into(),
            role: "Panel".into(),
            surface_variant: surface_variant.into(),
            corner_radius: 6.0,
            frame: TemplateNodeFrameData {
                x: 12.0,
                y: 8.0,
                width: 74.0,
                height: 42.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn placeholder_rect() -> FrameRect {
        FrameRect {
            x: 12.0,
            y: 8.0,
            width: 74.0,
            height: 42.0,
        }
    }

    fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y as usize * frame_width as usize) + x as usize) * 4;
        [
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]
    }
}
