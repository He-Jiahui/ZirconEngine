use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::material_state_layer::push_state_layer_commands;
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::{
    border_color, draws_elevation_shadow, elevation_shadow_rect, surface_color,
    template_border_width, template_corner_radius,
};
use super::eligibility::draws_border;
use crate::ui::retained_host::host_contract::paint_geometry::{corner_radius_for_frame, intersect};

const ASSET_THUMBNAIL_NAME_AREA_SURFACE: &str = "asset-thumbnail-name-area";
const MATERIAL_ELEVATION_SHADOW_OPACITY: f32 = 0.72;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_surface_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !has_paintable_surface_extent(rect) || intersect(rect, clip).is_none() {
        return;
    }
    let border_width = template_border_width(node);
    let corner_radius = corner_radius_for_frame(rect, template_corner_radius(node));
    let draws_asset_thumbnail_name_area =
        draws_asset_thumbnail_name_area_surface(node, corner_radius);
    if draws_elevation_shadow(node) {
        let shadow_rect = elevation_shadow_rect(rect, node.elevation);
        commands.push(
            HostPaintCommand::quad(
                shadow_rect.clone(),
                Some(clip.clone()),
                order - 1,
                Some(PALETTE.shadow),
                None,
                0.0,
                corner_radius,
                MATERIAL_ELEVATION_SHADOW_OPACITY * opacity,
            )
            .with_box_shadow(
                shadow_rect.x - rect.x,
                shadow_rect.y - rect.y,
                0.0,
                0.0,
                corner_radius,
                false,
            ),
        );
    }
    if draws_asset_thumbnail_name_area {
        push_asset_thumbnail_name_area_surface_commands(
            commands,
            node,
            rect,
            clip,
            order,
            opacity,
            border_width,
            corner_radius,
        );
    } else {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(surface_color(node)),
            draws_border(node).then_some(border_color(node)),
            border_width,
            corner_radius,
            opacity,
        ));
    }
    if draws_asset_thumbnail_name_area {
        push_asset_thumbnail_name_area_state_layer_commands(
            commands,
            node,
            rect,
            clip,
            corner_radius,
            order + 1,
            opacity,
        );
    } else {
        push_state_layer_commands(
            commands,
            node,
            rect,
            clip,
            corner_radius,
            order + 1,
            opacity,
        );
    }
}

fn has_paintable_surface_extent(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

fn draws_asset_thumbnail_name_area_surface(
    node: &TemplatePaneNodeData,
    corner_radius: f32,
) -> bool {
    node.surface_variant.as_str() == ASSET_THUMBNAIL_NAME_AREA_SURFACE && corner_radius > 0.0
}

fn push_asset_thumbnail_name_area_surface_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    border_width: f32,
    corner_radius: f32,
) {
    let fill = surface_color(node);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(fill),
        draws_border(node).then_some(border_color(node)),
        border_width,
        corner_radius,
        opacity,
    ));

    if let Some(top_cap) = asset_thumbnail_name_area_square_top_cap(rect, corner_radius) {
        commands.push(HostPaintCommand::quad(
            top_cap,
            Some(clip.clone()),
            order,
            Some(fill),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn asset_thumbnail_name_area_square_top_cap(
    rect: &FrameRect,
    corner_radius: f32,
) -> Option<FrameRect> {
    let height = corner_radius.min(rect.height).max(0.0);
    if rect.width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some(FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height,
    })
}

fn push_asset_thumbnail_name_area_state_layer_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    corner_radius: f32,
    order: i32,
    opacity: f32,
) {
    let command_count_before_state_layer = commands.len();
    push_state_layer_commands(commands, node, rect, clip, corner_radius, order, opacity);

    let Some(top_cap) = asset_thumbnail_name_area_square_top_cap(rect, corner_radius) else {
        return;
    };
    let Some(state_layer) = commands[command_count_before_state_layer..]
        .iter()
        .find(|command| {
            command.z_index == order
                && command.frame == *rect
                && command.corner_radius == corner_radius
                && command.border_width == 0.0
        })
    else {
        return;
    };
    let state_layer_background = state_layer.background_color;
    let state_layer_opacity = state_layer.opacity;
    commands.push(HostPaintCommand::quad(
        top_cap,
        Some(clip.clone()),
        order,
        state_layer_background,
        None,
        0.0,
        0.0,
        state_layer_opacity,
    ));
}

#[cfg(test)]
#[path = "commands/tests/cached_asset_surface_tests.rs"]
mod cached_asset_surface_tests;

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
