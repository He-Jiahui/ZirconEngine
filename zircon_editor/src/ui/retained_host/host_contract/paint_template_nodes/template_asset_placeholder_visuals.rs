use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;
use super::visual_assets::load_existing_icon_asset_pixels_for_size;

const VISUAL_SURFACE_INSET_RATIO: f32 = 0.12;
const VISUAL_SURFACE_MIN_INSET: f32 = 5.0;
const VISUAL_SURFACE_MAX_INSET: f32 = 8.0;
const VISUAL_SURFACE_RADIUS: f32 = 4.0;
const ASSET_THUMBNAIL_VISUAL_ROLE: &str = "asset-thumbnail-visual";
const DEFAULT_THUMBNAIL_ICON_NAME: &str = "image";
const THUMBNAIL_ICON_EDGE: u32 = 20;
const TYPED_THUMBNAIL_ICON_EDGE: u32 = 14;
const TYPED_THUMBNAIL_PLATE_PADDING: f32 = 4.0;
const TYPED_THUMBNAIL_PLATE_RADIUS: f32 = 4.0;
const TYPED_THUMBNAIL_BADGE_MARGIN: f32 = 3.0;
const TYPED_THUMBNAIL_PREVIEW_INSET: f32 = 4.0;
const TYPED_THUMBNAIL_PREVIEW_RADIUS: f32 = 3.0;
const TYPED_THUMBNAIL_MARK_RADIUS: f32 = 1.5;
const THUMBNAIL_TYPE_TEXTURE_TINT: [u8; 4] = [101, 174, 213, 255];
const THUMBNAIL_TYPE_MATERIAL_TINT: [u8; 4] = [211, 166, 83, 255];
const THUMBNAIL_TYPE_SCENE_TINT: [u8; 4] = [97, 190, 162, 255];
const THUMBNAIL_TYPE_MESH_TINT: [u8; 4] = [165, 177, 190, 255];
const THUMBNAIL_TYPE_SHADER_TINT: [u8; 4] = [125, 196, 132, 255];
const THUMBNAIL_TYPE_AUDIO_TINT: [u8; 4] = [110, 163, 220, 255];
const THUMBNAIL_TYPE_UI_TINT: [u8; 4] = [82, 186, 202, 255];
const THUMBNAIL_TYPE_DEFAULT_TINT: [u8; 4] = [174, 184, 194, 255];

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
        thumbnail_well_border_paint(node),
        thumbnail_well_border_width(node),
        VISUAL_SURFACE_RADIUS,
        opacity,
    ));

    if is_typed_thumbnail_visual(node) {
        push_typed_thumbnail_preview_commands(
            commands,
            node,
            &inner_rect,
            clip,
            order + 1,
            opacity,
        );
        push_thumbnail_icon_plate_command(commands, node, &inner_rect, clip, order + 5, opacity);
        push_thumbnail_icon_command(commands, node, &inner_rect, clip, order + 6, opacity);
    } else {
        push_thumbnail_icon_command(commands, node, &inner_rect, clip, order + 1, opacity);
    }
}

fn is_asset_thumbnail_visual(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.surface_variant.as_str(),
        "asset-placeholder-visual" | "asset-preview-visual"
    )
}

fn thumbnail_well_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if is_typed_thumbnail_visual(node) {
        return PALETTE.surface_disabled;
    }
    match node.surface_variant.as_str() {
        "asset-preview-visual" => PALETTE.surface_inset,
        _ => PALETTE.surface,
    }
}

fn thumbnail_well_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if !is_typed_thumbnail_visual(node) {
        return [0, 0, 0, 0];
    }
    if node.focused {
        PALETTE.focus_ring
    } else {
        PALETTE.separator_soft
    }
}

fn thumbnail_well_border_paint(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    is_typed_thumbnail_visual(node).then_some(thumbnail_well_border_color(node))
}

fn thumbnail_well_border_width(node: &TemplatePaneNodeData) -> f32 {
    if is_typed_thumbnail_visual(node) {
        1.0
    } else {
        0.0
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
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(edge) = thumbnail_icon_edge(node, rect) else {
        return;
    };
    let Some(image) = load_existing_icon_asset_pixels_for_size(
        thumbnail_icon_name(node),
        edge,
        edge,
        thumbnail_icon_tint(node),
    ) else {
        return;
    };

    commands.push(HostPaintCommand::image_pixels(
        thumbnail_icon_rect(node, rect, edge),
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

fn push_thumbnail_icon_plate_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !is_typed_thumbnail_visual(node) {
        return;
    }
    let Some(edge) = thumbnail_icon_edge(node, rect) else {
        return;
    };
    let plate_rect = thumbnail_icon_plate_rect(rect, edge);
    commands.push(HostPaintCommand::quad(
        plate_rect,
        Some(clip.clone()),
        order,
        Some(PALETTE.surface_inset),
        Some(PALETTE.separator_soft),
        1.0,
        TYPED_THUMBNAIL_PLATE_RADIUS,
        opacity,
    ));
}

fn push_typed_thumbnail_preview_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(canvas) = typed_thumbnail_preview_rect(rect) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        canvas.clone(),
        Some(clip.clone()),
        order,
        Some(PALETTE.surface_inset),
        Some(PALETTE.separator_soft),
        1.0,
        TYPED_THUMBNAIL_PREVIEW_RADIUS,
        opacity,
    ));
    push_typed_thumbnail_content_marks(commands, node, &canvas, clip, order + 1, opacity);
}

fn push_typed_thumbnail_content_marks(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    canvas: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    match node.component_variant.as_str() {
        "asset-ui-layout" | "asset-ui-widget" | "asset-ui-style" => {
            push_ui_thumbnail_marks(commands, canvas, clip, order, opacity)
        }
        "asset-texture" => {
            push_texture_thumbnail_marks(commands, node, canvas, clip, order, opacity)
        }
        "asset-material" => {
            push_material_thumbnail_marks(commands, node, canvas, clip, order, opacity)
        }
        "asset-scene" | "asset-mesh" | "asset-prefab" => {
            push_scene_thumbnail_marks(commands, node, canvas, clip, order, opacity)
        }
        _ => push_generic_thumbnail_marks(commands, node, canvas, clip, order, opacity),
    }
}

fn push_ui_thumbnail_marks(
    commands: &mut Vec<HostPaintCommand>,
    canvas: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let inset = 5.0_f32.min(canvas.width * 0.16).min(canvas.height * 0.2);
    let content_x = canvas.x + inset;
    let content_y = canvas.y + inset;
    let content_w = (canvas.width - inset * 2.0).max(0.0);
    let accent_h = 3.0_f32.min(canvas.height * 0.14);
    push_thumbnail_mark(
        commands,
        FrameRect {
            x: content_x,
            y: content_y,
            width: content_w,
            height: accent_h,
        },
        thumbnail_type_tint_for_ui(),
        clip,
        order,
        opacity,
    );
    push_thumbnail_mark(
        commands,
        FrameRect {
            x: content_x,
            y: content_y + accent_h + 4.0,
            width: (content_w * 0.34).max(0.0),
            height: (canvas.height - inset * 2.0 - accent_h - 4.0).max(0.0),
        },
        PALETTE.surface,
        clip,
        order + 1,
        opacity,
    );
    let line_x = content_x + content_w * 0.42;
    let line_w = (content_w * 0.42).max(0.0);
    for row in 0..2 {
        push_thumbnail_mark(
            commands,
            FrameRect {
                x: line_x,
                y: content_y + accent_h + 5.0 + row as f32 * 6.0,
                width: line_w,
                height: 2.0,
            },
            PALETTE.separator_soft,
            clip,
            order + 2 + row,
            opacity,
        );
    }
}

fn push_texture_thumbnail_marks(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    canvas: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let cell_w = (canvas.width - 8.0).max(0.0) * 0.5;
    let cell_h = (canvas.height - 8.0).max(0.0) * 0.5;
    for row in 0..2 {
        for col in 0..2 {
            let is_tinted = (row + col) % 2 == 0;
            push_thumbnail_mark(
                commands,
                FrameRect {
                    x: canvas.x + 4.0 + col as f32 * cell_w,
                    y: canvas.y + 4.0 + row as f32 * cell_h,
                    width: cell_w,
                    height: cell_h,
                },
                if is_tinted {
                    thumbnail_type_tint(node)
                } else {
                    PALETTE.surface
                },
                clip,
                order + row * 2 + col,
                opacity,
            );
        }
    }
}

fn push_material_thumbnail_marks(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    canvas: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let swatch_w = (canvas.width - 14.0).max(0.0) * 0.55;
    let swatch_h = (canvas.height - 12.0).max(0.0);
    push_thumbnail_mark(
        commands,
        FrameRect {
            x: canvas.x + 5.0,
            y: canvas.y + 6.0,
            width: swatch_w,
            height: swatch_h,
        },
        thumbnail_type_tint(node),
        clip,
        order,
        opacity,
    );
    for row in 0..3 {
        push_thumbnail_mark(
            commands,
            FrameRect {
                x: canvas.x + swatch_w + 9.0,
                y: canvas.y + 7.0 + row as f32 * 6.0,
                width: (canvas.width - swatch_w - 15.0).max(0.0),
                height: 2.0,
            },
            PALETTE.separator_soft,
            clip,
            order + 1 + row,
            opacity,
        );
    }
}

fn push_scene_thumbnail_marks(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    canvas: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    for index in 0..3 {
        let x = canvas.x + 5.0 + index as f32 * ((canvas.width - 12.0) / 3.0).max(0.0);
        let y = canvas.y + 7.0 + (index % 2) as f32 * 7.0;
        push_thumbnail_mark(
            commands,
            FrameRect {
                x,
                y,
                width: 8.0_f32.min(canvas.width * 0.18),
                height: 8.0_f32.min(canvas.height * 0.34),
            },
            if index == 1 {
                thumbnail_type_tint(node)
            } else {
                PALETTE.surface
            },
            clip,
            order + index,
            opacity,
        );
    }
    push_thumbnail_mark(
        commands,
        FrameRect {
            x: canvas.x + 5.0,
            y: canvas.y + canvas.height - 7.0,
            width: (canvas.width - 10.0).max(0.0),
            height: 1.5,
        },
        PALETTE.separator_soft,
        clip,
        order + 3,
        opacity,
    );
}

fn push_generic_thumbnail_marks(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    canvas: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_thumbnail_mark(
        commands,
        FrameRect {
            x: canvas.x + 5.0,
            y: canvas.y + 6.0,
            width: (canvas.width - 10.0).max(0.0),
            height: 3.0,
        },
        thumbnail_type_tint(node),
        clip,
        order,
        opacity,
    );
    for row in 0..2 {
        push_thumbnail_mark(
            commands,
            FrameRect {
                x: canvas.x + 6.0,
                y: canvas.y + 14.0 + row as f32 * 6.0,
                width: (canvas.width * 0.58).max(0.0),
                height: 2.0,
            },
            PALETTE.separator_soft,
            clip,
            order + 1 + row,
            opacity,
        );
    }
}

fn push_thumbnail_mark(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    color: [u8; 4],
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        TYPED_THUMBNAIL_MARK_RADIUS,
        opacity,
    ));
}

fn thumbnail_icon_name(node: &TemplatePaneNodeData) -> &str {
    if node.component_role.as_str() == ASSET_THUMBNAIL_VISUAL_ROLE {
        let component_icon = node.component_variant.trim();
        if !component_icon.is_empty() {
            return component_icon;
        }
    }
    if !node.icon_name.trim().is_empty() {
        return node.icon_name.as_str();
    }
    DEFAULT_THUMBNAIL_ICON_NAME
}

fn is_typed_thumbnail_visual(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == ASSET_THUMBNAIL_VISUAL_ROLE
        && !node.component_variant.trim().is_empty()
}

fn thumbnail_icon_tint(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    (!is_typed_thumbnail_visual(node)).then_some(PALETTE.text_disabled)
}

fn thumbnail_type_tint(node: &TemplatePaneNodeData) -> [u8; 4] {
    match node.component_variant.as_str() {
        "asset-texture" => THUMBNAIL_TYPE_TEXTURE_TINT,
        "asset-material" => THUMBNAIL_TYPE_MATERIAL_TINT,
        "asset-scene" => THUMBNAIL_TYPE_SCENE_TINT,
        "asset-mesh" | "asset-prefab" => THUMBNAIL_TYPE_MESH_TINT,
        "asset-shader" => THUMBNAIL_TYPE_SHADER_TINT,
        "asset-audio" => THUMBNAIL_TYPE_AUDIO_TINT,
        "asset-ui-layout" | "asset-ui-widget" | "asset-ui-style" => THUMBNAIL_TYPE_UI_TINT,
        _ => THUMBNAIL_TYPE_DEFAULT_TINT,
    }
}

fn thumbnail_type_tint_for_ui() -> [u8; 4] {
    THUMBNAIL_TYPE_UI_TINT
}

fn thumbnail_icon_edge(node: &TemplatePaneNodeData, rect: &FrameRect) -> Option<u32> {
    let max_edge = rect.width.min(rect.height).floor() as u32;
    let desired_edge = if is_typed_thumbnail_visual(node) {
        TYPED_THUMBNAIL_ICON_EDGE
    } else {
        THUMBNAIL_ICON_EDGE
    };
    let edge = desired_edge.min(max_edge);
    (edge > 0).then_some(edge)
}

fn typed_thumbnail_preview_rect(rect: &FrameRect) -> Option<FrameRect> {
    let width = (rect.width - TYPED_THUMBNAIL_PREVIEW_INSET * 2.0).max(0.0);
    let height = (rect.height - TYPED_THUMBNAIL_PREVIEW_INSET * 2.0).max(0.0);
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some(FrameRect {
        x: rect.x + TYPED_THUMBNAIL_PREVIEW_INSET,
        y: rect.y + TYPED_THUMBNAIL_PREVIEW_INSET,
        width,
        height,
    })
}

fn thumbnail_icon_rect(node: &TemplatePaneNodeData, rect: &FrameRect, edge: u32) -> FrameRect {
    let edge = edge as f32;
    if is_typed_thumbnail_visual(node) {
        let plate = thumbnail_icon_plate_rect(rect, edge as u32);
        return FrameRect {
            x: plate.x + (plate.width - edge) * 0.5,
            y: plate.y + (plate.height - edge) * 0.5,
            width: edge,
            height: edge,
        };
    }
    FrameRect {
        x: rect.x + (rect.width - edge) * 0.5,
        y: rect.y + (rect.height - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

fn thumbnail_icon_plate_rect(rect: &FrameRect, icon_edge: u32) -> FrameRect {
    let edge = (icon_edge as f32 + TYPED_THUMBNAIL_PLATE_PADDING * 2.0)
        .min(rect.width)
        .min(rect.height);
    if edge < rect.width && edge < rect.height {
        return FrameRect {
            x: rect.x + rect.width - edge - TYPED_THUMBNAIL_BADGE_MARGIN,
            y: rect.y + TYPED_THUMBNAIL_BADGE_MARGIN,
            width: edge,
            height: edge,
        };
    }
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
    fn asset_placeholder_visual_uses_declared_asset_type_icon() {
        let rect = placeholder_rect();
        let mut commands = Vec::new();
        let mut node = placeholder_node("asset-placeholder-visual");
        node.component_role = "asset-thumbnail-visual".into();
        node.component_variant = "asset-texture".into();

        push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

        let icon = commands
            .iter()
            .find_map(|command| command.image_pixels.as_ref())
            .expect("asset thumbnail visual should paint a type-specific icon");
        assert_eq!((icon.width, icon.height), (14, 14));
        assert!(
            !icon.resource_key.starts_with("missing-icon:"),
            "asset thumbnail icon should resolve through a real texture asset SVG, got {}",
            icon.resource_key
        );
        assert!(
            visible_color_count(&icon.rgba) > 3,
            "typed thumbnail icons should preserve the source SVG colors instead of flattening to one tint"
        );
    }

    #[test]
    fn asset_thumbnail_visual_uses_muted_corner_icon_plate_without_type_border() {
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
        assert!(paint_commands.len() >= 6);
        assert_eq!(
            paint_commands[0].background_color,
            Some(PALETTE.surface_disabled)
        );
        assert_eq!(paint_commands[0].border_color, Some(PALETTE.separator_soft));
        assert_eq!(paint_commands[0].border_width, 1.0);

        let Some(badge_plate) = paint_commands.last() else {
            assert!(
                false,
                "typed thumbnail visual should paint a muted corner icon plate"
            );
            return;
        };
        assert_eq!(badge_plate.background_color, Some(PALETTE.surface_inset));
        assert_eq!(badge_plate.border_color, Some(PALETTE.separator_soft));
        assert_eq!(badge_plate.corner_radius, TYPED_THUMBNAIL_PLATE_RADIUS);
        assert!(
            badge_plate.frame.x > rect.x + rect.width * 0.5,
            "semantic icon plate should sit in the corner instead of the thumbnail center"
        );
        assert!(
            paint_commands[2..paint_commands.len() - 1]
                .iter()
                .any(|command| command.background_color == Some(THUMBNAIL_TYPE_UI_TINT)),
            "asset type tint should stay in thumbnail content marks, not the corner icon plate border"
        );
    }

    #[test]
    fn asset_thumbnail_visual_uses_muted_preview_canvas_border_and_corner_type_icon() {
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
        assert!(
            paint_commands.len() >= 5,
            "typed asset thumbnail should paint a preview canvas and content marks before the icon"
        );
        assert_eq!(
            paint_commands[0].background_color,
            Some(PALETTE.surface_disabled)
        );
        assert_eq!(
            paint_commands[1].background_color,
            Some(PALETTE.surface_inset)
        );
        assert_eq!(paint_commands[1].border_color, Some(PALETTE.separator_soft));
        assert!(
            paint_commands[2..]
                .iter()
                .any(|command| command.background_color == Some(THUMBNAIL_TYPE_UI_TINT)),
            "typed preview should include an asset-color content mark"
        );

        let icon_command = commands
            .iter()
            .find(|command| command.image_pixels.is_some())
            .expect("typed thumbnail should still paint the semantic icon");
        assert!(
            icon_command.frame.x > rect.x + rect.width * 0.5,
            "semantic icon should move to a corner badge instead of dominating the preview"
        );
        assert!(icon_command.frame.width <= 16.0);
        assert!(icon_command.frame.height <= 16.0);
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
        assert_eq!(well.border_color, Some(PALETTE.separator_soft));

        commands.clear();
        node.focused = true;
        push_asset_placeholder_visual_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

        let focused_well = commands
            .iter()
            .find(|command| command.image_pixels.is_none())
            .expect("focused typed thumbnail visual should paint a well");
        assert_eq!(focused_well.border_color, Some(PALETTE.focus_ring));
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
}
