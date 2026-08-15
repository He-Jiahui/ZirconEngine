use super::super::template_node_pipeline::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};
use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};

#[test]
fn asset_thumbnail_visual_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.radius_control = 3.0;
    host.border_width = 1.5;
    host.gap_s = 5.0;
    host.gap_m = 11.0;
    host.gap_l = 13.0;
    host.row_height = 27.0;

    let metrics = asset_visual_metrics_from_host(host);

    assert_eq!(metrics.surface_radius, 3.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.visual_surface_min_inset, 6.5);
    assert_eq!(metrics.visual_surface_max_inset, 11.0);
    assert_eq!(metrics.typed_surface_min_inset, 3.5);
    assert_eq!(metrics.typed_surface_max_inset, 5.0);
    assert_eq!(metrics.icon_min_edge, 27);
    assert_eq!(metrics.icon_max_edge, 38);
    assert_eq!(metrics.typed_preview_icon_min_edge, 38);
    assert_eq!(metrics.typed_preview_icon_max_edge, 53);
}

#[test]
fn asset_thumbnail_visual_skips_collapsed_or_fully_clipped_roots() {
    let node = placeholder_node("asset-placeholder-visual");
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 112.0,
        height: 64.0,
    };
    let mut commands = Vec::new();

    push_asset_placeholder_visual_commands(
        &mut commands,
        &node,
        &FrameRect {
            width: 0.0,
            ..placeholder_rect()
        },
        &clip,
        0,
        1.0,
    );
    assert!(commands.is_empty());

    push_asset_placeholder_visual_commands(
        &mut commands,
        &node,
        &placeholder_rect(),
        &FrameRect {
            x: 88.0,
            width: 32.0,
            ..clip
        },
        0,
        1.0,
    );
    assert!(commands.is_empty());
}

#[test]
fn asset_thumbnail_visual_partially_clipped_root_keeps_clipped_commands() {
    let node = placeholder_node("asset-placeholder-visual");
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 32.0,
        height: 64.0,
    };
    let mut commands = Vec::new();

    push_asset_placeholder_visual_commands(
        &mut commands,
        &node,
        &placeholder_rect(),
        &clip,
        0,
        1.0,
    );

    assert!(!commands.is_empty());
    assert!(commands
        .iter()
        .all(|command| command.clip_frame.as_ref() == Some(&clip)));
}

#[test]
fn asset_thumbnail_visual_palette_projects_from_host_material_palette() {
    let mut host = PALETTE;
    host.surface = [1, 2, 3, 4];
    host.surface_inset = [5, 6, 7, 8];
    host.separator_soft = [9, 10, 11, 12];
    host.focus_ring = [13, 14, 15, 16];
    host.text_muted = [17, 18, 19, 20];

    let palette = asset_visual_palette_from_host(host);

    assert_eq!(palette.placeholder_well, [1, 2, 3, 4]);
    assert_eq!(palette.preview_well, [5, 6, 7, 8]);
    assert_eq!(palette.typed_well, [5, 6, 7, 8]);
    assert_eq!(palette.typed_border, [9, 10, 11, 12]);
    assert_eq!(palette.focused_border, [13, 14, 15, 16]);
    assert_eq!(palette.placeholder_icon_tint, [17, 18, 19, 20]);
}

#[test]
fn asset_placeholder_visual_uses_single_recessed_well_and_relative_svg_icon() {
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
    assert_eq!(
        well_commands[0].background_color,
        Some(asset_visual_palette_from_host(PALETTE).placeholder_well)
    );
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
    let metrics = asset_visual_metrics_from_host(METRICS);
    assert_eq!(
        (icon.width, icon.height),
        (metrics.icon_min_edge, metrics.icon_min_edge)
    );
    assert!(
        !icon.resource_key.starts_with("missing-icon:"),
        "placeholder icon should resolve through the real shell image icon"
    );
    assert!(
        icon_commands[0].frame.width > 20.0,
        "generic image preview should scale from the well instead of staying at the old fixed 20px size"
    );
    assert_eq!(icon_commands[0].frame.width, metrics.icon_min_edge as f32);
    assert_eq!(icon_commands[0].frame.height, metrics.icon_min_edge as f32);
}

#[test]
fn asset_placeholder_visual_uses_declared_asset_type_icon() {
    let rect = placeholder_rect();
    let mut commands = Vec::new();
    let mut node = placeholder_node("asset-placeholder-visual");
    node.component_role = "asset-thumbnail-visual".into();
    node.component_variant = "asset-texture".into();

    push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let icon_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        icon_commands.len(),
        1,
        "typed thumbnail visual should paint one primary preview icon; type identity is already carried by the tile metadata"
    );

    let preview_icon = icon_commands[0]
        .image_pixels
        .as_ref()
        .expect("preview icon should carry real SVG pixels");
    let metrics = asset_visual_metrics_from_host(METRICS);
    assert!(
        preview_icon.width >= metrics.typed_preview_icon_min_edge,
        "preview icon should be large enough to carry asset identity in dense tiles"
    );
    assert!(
        !preview_icon.resource_key.starts_with("missing-icon:"),
        "asset thumbnail preview icon should resolve through a real texture asset SVG, got {}",
        preview_icon.resource_key
    );
    assert!(
        visible_color_count(&preview_icon.rgba) > 3,
        "typed thumbnail icons should preserve the source SVG colors instead of flattening to one tint"
    );
}

#[test]
fn asset_thumbnail_visual_uses_single_well_without_corner_icon_plate() {
    let rect = placeholder_rect();
    let mut commands = Vec::new();
    let mut node = placeholder_node("asset-placeholder-visual");
    node.component_role = "asset-thumbnail-visual".into();
    node.component_variant = "asset-ui-layout".into();

    push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let paint_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_none())
        .collect::<Vec<_>>();
    assert_eq!(
        paint_commands.len(),
        1,
        "typed thumbnail visual should avoid both abstract content mark quads and duplicate corner badge plates"
    );
    assert_eq!(
        paint_commands[0].background_color,
        Some(asset_visual_palette_from_host(PALETTE).typed_well)
    );
    assert_eq!(
        paint_commands[0].border_color,
        Some(asset_visual_palette_from_host(PALETTE).typed_border)
    );
    assert_eq!(
        paint_commands[0].border_width,
        asset_visual_metrics_from_host(METRICS).border_width
    );
    assert!(
        commands
            .iter()
            .filter(|command| command.image_pixels.is_some())
            .count()
            == 1,
        "semantic identity should be carried by the primary preview icon and type badge text, not by a duplicate corner badge"
    );
}

#[test]
fn asset_thumbnail_visual_uses_centered_semantic_preview_icon_without_corner_badge() {
    let rect = placeholder_rect();
    let mut commands = Vec::new();
    let mut node = placeholder_node("asset-placeholder-visual");
    node.component_role = "asset-thumbnail-visual".into();
    node.component_variant = "asset-ui-layout".into();

    push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let paint_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_none())
        .collect::<Vec<_>>();
    assert_eq!(
        paint_commands.len(),
        1,
        "typed asset thumbnail should paint one large preview well and no duplicate corner badge plate"
    );
    assert_eq!(
        paint_commands[0].background_color,
        Some(asset_visual_palette_from_host(PALETTE).typed_well)
    );
    assert_eq!(
        paint_commands[0].border_color,
        Some(asset_visual_palette_from_host(PALETTE).typed_border)
    );
    assert!(paint_commands[0].frame.width >= rect.width - 8.0);
    assert!(paint_commands[0].frame.height >= rect.height - 8.0);
    let icon_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        icon_commands.len(),
        1,
        "typed thumbnail should paint only the primary preview icon"
    );

    let preview_icon = icon_commands[0];
    let metrics = asset_visual_metrics_from_host(METRICS);
    assert!(preview_icon.frame.width >= metrics.typed_preview_icon_min_edge as f32);
    assert!(preview_icon.frame.height >= metrics.typed_preview_icon_min_edge as f32);
    assert!(preview_icon.frame.width <= metrics.typed_preview_icon_max_edge as f32);
    assert!(preview_icon.frame.height <= metrics.typed_preview_icon_max_edge as f32);
    assert!(
        (preview_icon.frame.x + preview_icon.frame.width * 0.5
            - (paint_commands[0].frame.x + paint_commands[0].frame.width * 0.5))
            .abs()
            <= 1.0,
        "preview icon should be centered in the thumbnail well after removing the corner badge"
    );
    assert!(
        (preview_icon.frame.y + preview_icon.frame.height * 0.5
            - (paint_commands[0].frame.y + paint_commands[0].frame.height * 0.5))
            .abs()
            <= 1.0,
        "preview icon should be vertically centered in the thumbnail well"
    );
}

#[test]
fn asset_thumbnail_visual_scales_semantic_icon_to_slate_tile_max() {
    let mut rect = large_thumbnail_rect();
    rect.height = 120.0;
    let mut commands = Vec::new();
    let mut node = placeholder_node("asset-placeholder-visual");
    node.component_role = "asset-thumbnail-visual".into();
    node.component_variant = "asset-scene".into();

    push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let preview_icon = commands
        .iter()
        .find(|command| command.image_pixels.is_some())
        .expect("large typed thumbnail should paint a semantic preview icon");

    assert_eq!(
        preview_icon.frame.width,
        asset_visual_metrics_from_host(METRICS).typed_preview_icon_max_edge as f32
    );
    assert_eq!(
        preview_icon.frame.height,
        asset_visual_metrics_from_host(METRICS).typed_preview_icon_max_edge as f32
    );
}

#[test]
fn asset_thumbnail_visual_prefers_projected_preview_image_over_semantic_icon() {
    let rect = placeholder_rect();
    let mut commands = Vec::new();
    let mut node = placeholder_node("asset-preview-visual");
    node.component_role = "asset-thumbnail-visual".into();
    node.component_variant = "asset-texture".into();
    node.media_source = "ui/editor/showcase_checker.svg".into();
    node.has_preview_image = true;
    node.preview_image = solid_preview_image([201, 42, 33, 255]);

    push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let image_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        image_commands.len(),
        1,
        "real thumbnail preview should replace the semantic fallback icon, not add beside it"
    );
    let preview = image_commands[0]
        .image_pixels
        .as_ref()
        .expect("asset preview command should carry retained preview pixels");
    assert_eq!((preview.width, preview.height), (2, 2));
    assert_eq!(&preview.rgba[0..4], &[201, 42, 33, 255]);
    assert!(
        image_commands[0].frame.width
            > asset_visual_metrics_from_host(METRICS).icon_min_edge as f32,
        "projected thumbnails should use the preview well instead of the generic icon size"
    );
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

#[test]
fn asset_preview_thumbnail_visual_keeps_muted_well_border_until_focused() {
    let rect = placeholder_rect();
    let mut commands = Vec::new();
    let mut node = placeholder_node("asset-preview-visual");
    node.component_role = "asset-thumbnail-visual".into();
    node.component_variant = "asset-material".into();

    push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let well = commands
        .iter()
        .find(|command| command.image_pixels.is_none())
        .expect("typed thumbnail visual should paint a well");
    assert_eq!(
        well.border_color,
        Some(asset_visual_palette_from_host(PALETTE).typed_border)
    );

    commands.clear();
    node.focused = true;
    push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    let focused_well = commands
        .iter()
        .find(|command| command.image_pixels.is_none())
        .expect("focused typed thumbnail visual should paint a well");
    assert_eq!(
        focused_well.border_color,
        Some(asset_visual_palette_from_host(PALETTE).focused_border)
    );
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

fn large_thumbnail_rect() -> FrameRect {
    FrameRect {
        x: 8.0,
        y: 8.0,
        width: 108.0,
        height: 88.0,
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

fn visible_color_count(rgba: &[u8]) -> usize {
    let mut colors = Vec::new();
    for pixel in rgba.chunks_exact(4) {
        if pixel[3] == 0 {
            continue;
        }
        let color = [pixel[0], pixel[1], pixel[2], pixel[3]];
        if !colors.contains(&color) {
            colors.push(color);
        }
    }
    colors.len()
}

fn solid_preview_image(color: [u8; 4]) -> Image {
    let pixels = [color, color, color, color].concat();
    Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &pixels, 2, 2,
    ))
}
