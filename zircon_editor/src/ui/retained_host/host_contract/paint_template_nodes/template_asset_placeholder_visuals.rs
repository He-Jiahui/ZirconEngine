use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};
use super::render_commands::HostPaintCommand;
use super::visual_assets::load_existing_icon_asset_pixels_for_size;

mod geometry;
mod preview_image;

use self::geometry::{has_paintable_thumbnail_extent, thumbnail_surface_rect};
use self::preview_image::push_thumbnail_preview_image_command;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

const VISUAL_SURFACE_INSET_RATIO: f32 = 0.12;
const TYPED_THUMBNAIL_SURFACE_INSET_RATIO: f32 = 0.055;
const ASSET_THUMBNAIL_VISUAL_ROLE: &str = "asset-thumbnail-visual";
const DEFAULT_THUMBNAIL_ICON_NAME: &str = "image";
const THUMBNAIL_ICON_RATIO: f32 = 0.64;
const TYPED_THUMBNAIL_PREVIEW_ICON_RATIO: f32 = 0.62;

#[derive(Clone, Copy, Debug, PartialEq)]
struct WorkbenchAssetVisualMetrics {
    visual_surface_min_inset: f32,
    visual_surface_max_inset: f32,
    typed_surface_min_inset: f32,
    typed_surface_max_inset: f32,
    surface_radius: f32,
    border_width: f32,
    icon_min_edge: u32,
    icon_max_edge: u32,
    typed_preview_icon_min_edge: u32,
    typed_preview_icon_max_edge: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorkbenchAssetVisualPalette {
    placeholder_well: [u8; 4],
    preview_well: [u8; 4],
    typed_well: [u8; 4],
    typed_border: [u8; 4],
    focused_border: [u8; 4],
    placeholder_icon_tint: [u8; 4],
}

fn asset_visual_metrics() -> WorkbenchAssetVisualMetrics {
    asset_visual_metrics_from_host(current_host_metrics())
}

fn asset_visual_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchAssetVisualMetrics {
    let visual_surface_min_inset = (metrics.gap_s + metrics.border_width).max(metrics.border_width);
    let visual_surface_max_inset = metrics.gap_m.max(visual_surface_min_inset);
    let typed_surface_min_inset = (metrics.gap_s - metrics.border_width).max(metrics.border_width);
    let typed_surface_max_inset = metrics.gap_s.max(typed_surface_min_inset);
    let icon_min_edge = metric_edge(metrics.row_height);
    let icon_max_edge = metric_edge(metrics.row_height + metrics.gap_m).max(icon_min_edge);

    let typed_preview_icon_min_edge =
        metric_edge(metrics.row_height + metrics.gap_m).max(icon_min_edge);
    let typed_preview_icon_max_edge =
        metric_edge(metrics.row_height + metrics.gap_l * 2.0).max(typed_preview_icon_min_edge);

    WorkbenchAssetVisualMetrics {
        visual_surface_min_inset,
        visual_surface_max_inset,
        typed_surface_min_inset,
        typed_surface_max_inset,
        surface_radius: metrics.radius_control,
        border_width: metrics.border_width,
        icon_min_edge,
        icon_max_edge,
        typed_preview_icon_min_edge,
        typed_preview_icon_max_edge,
    }
}

fn asset_visual_palette() -> WorkbenchAssetVisualPalette {
    asset_visual_palette_from_host(current_host_palette())
}

fn asset_visual_palette_from_host(palette: HostMaterialPalette) -> WorkbenchAssetVisualPalette {
    WorkbenchAssetVisualPalette {
        placeholder_well: palette.surface,
        preview_well: palette.surface_inset,
        typed_well: palette.surface_inset,
        typed_border: palette.separator_soft,
        focused_border: palette.focus_ring,
        placeholder_icon_tint: palette.text_muted,
    }
}

fn metric_edge(value: f32) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.round() as u32
}

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
    if !has_paintable_thumbnail_extent(rect) || intersect(rect, clip).is_none() {
        return;
    }

    let metrics = asset_visual_metrics();
    let palette = asset_visual_palette();
    let Some(inner_rect) = thumbnail_surface_rect(node, rect, metrics) else {
        return;
    };
    if intersect(&inner_rect, clip).is_none() {
        return;
    }

    commands.push(HostPaintCommand::quad(
        inner_rect.clone(),
        Some(clip.clone()),
        order,
        Some(thumbnail_well_color(node, palette)),
        thumbnail_well_border_paint(node, palette),
        thumbnail_well_border_width(node, metrics),
        metrics.surface_radius,
        opacity,
    ));

    if push_thumbnail_preview_image_command(commands, node, &inner_rect, clip, order + 1, opacity) {
        return;
    }

    if is_typed_thumbnail_visual(node) {
        push_typed_thumbnail_preview_commands(
            commands,
            node,
            &inner_rect,
            clip,
            order + 1,
            opacity,
            metrics,
            palette,
        );
    } else {
        push_thumbnail_icon_command(
            commands,
            node,
            &inner_rect,
            clip,
            order + 1,
            opacity,
            metrics,
            palette,
        );
    }
}

fn is_asset_thumbnail_visual(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.surface_variant.as_str(),
        "asset-placeholder-visual" | "asset-preview-visual"
    )
}

fn thumbnail_well_color(
    node: &TemplatePaneNodeData,
    palette: WorkbenchAssetVisualPalette,
) -> [u8; 4] {
    if is_typed_thumbnail_visual(node) {
        return palette.typed_well;
    }
    match node.surface_variant.as_str() {
        "asset-preview-visual" => palette.preview_well,
        _ => palette.placeholder_well,
    }
}

fn thumbnail_well_border_color(
    node: &TemplatePaneNodeData,
    palette: WorkbenchAssetVisualPalette,
) -> [u8; 4] {
    if !is_typed_thumbnail_visual(node) {
        return [0, 0, 0, 0];
    }
    if node.focused {
        palette.focused_border
    } else {
        palette.typed_border
    }
}

fn thumbnail_well_border_paint(
    node: &TemplatePaneNodeData,
    palette: WorkbenchAssetVisualPalette,
) -> Option<[u8; 4]> {
    is_typed_thumbnail_visual(node).then_some(thumbnail_well_border_color(node, palette))
}

fn thumbnail_well_border_width(
    node: &TemplatePaneNodeData,
    metrics: WorkbenchAssetVisualMetrics,
) -> f32 {
    if is_typed_thumbnail_visual(node) {
        metrics.border_width
    } else {
        0.0
    }
}

fn push_thumbnail_icon_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: WorkbenchAssetVisualMetrics,
    palette: WorkbenchAssetVisualPalette,
) {
    let Some(edge) = thumbnail_icon_edge(node, rect, metrics) else {
        return;
    };
    let Some(image) = load_existing_icon_asset_pixels_for_size(
        thumbnail_icon_name(node),
        edge,
        edge,
        thumbnail_icon_tint(node, palette),
    ) else {
        return;
    };

    let icon_rect = thumbnail_icon_rect(node, rect, edge);
    if intersect(&icon_rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::image_pixels(
        icon_rect,
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

fn push_typed_thumbnail_preview_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: WorkbenchAssetVisualMetrics,
    palette: WorkbenchAssetVisualPalette,
) {
    let Some(edge) = typed_thumbnail_preview_icon_edge(rect, metrics) else {
        return;
    };
    let Some(image) = load_existing_icon_asset_pixels_for_size(
        thumbnail_icon_name(node),
        edge,
        edge,
        thumbnail_icon_tint(node, palette),
    ) else {
        return;
    };

    let icon_rect = typed_thumbnail_preview_icon_rect(rect, edge);
    if intersect(&icon_rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::image_pixels(
        icon_rect,
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

fn thumbnail_icon_tint(
    node: &TemplatePaneNodeData,
    palette: WorkbenchAssetVisualPalette,
) -> Option<[u8; 4]> {
    (!is_typed_thumbnail_visual(node)).then_some(palette.placeholder_icon_tint)
}

fn thumbnail_icon_edge(
    _node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: WorkbenchAssetVisualMetrics,
) -> Option<u32> {
    let max_edge = rect.width.min(rect.height).floor() as u32;
    if max_edge == 0 {
        return None;
    }
    let desired_edge = ((max_edge as f32) * THUMBNAIL_ICON_RATIO).round() as u32;
    let edge = desired_edge
        .clamp(metrics.icon_min_edge, metrics.icon_max_edge)
        .min(max_edge);
    (edge > 0).then_some(edge)
}

fn typed_thumbnail_preview_icon_edge(
    rect: &FrameRect,
    metrics: WorkbenchAssetVisualMetrics,
) -> Option<u32> {
    let max_edge = rect.width.min(rect.height).floor() as u32;
    if max_edge == 0 {
        return None;
    }
    let desired_edge = ((max_edge as f32) * TYPED_THUMBNAIL_PREVIEW_ICON_RATIO).round() as u32;
    let edge = desired_edge
        .clamp(
            metrics.typed_preview_icon_min_edge,
            metrics.typed_preview_icon_max_edge,
        )
        .min(max_edge);
    (edge > 0).then_some(edge)
}

fn thumbnail_icon_rect(_node: &TemplatePaneNodeData, rect: &FrameRect, edge: u32) -> FrameRect {
    let edge = edge as f32;
    FrameRect {
        x: rect.x + (rect.width - edge) * 0.5,
        y: rect.y + (rect.height - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

fn typed_thumbnail_preview_icon_rect(rect: &FrameRect, edge: u32) -> FrameRect {
    let edge = edge as f32;
    FrameRect {
        x: rect.x + (rect.width - edge) * 0.5,
        y: rect.y + (rect.height - edge) * 0.5,
        width: edge,
        height: edge,
    }
}

#[cfg(test)]
mod tests;
