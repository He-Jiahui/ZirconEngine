use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::material_state_layer::push_state_layer_commands;
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::{
    border_color, draws_elevation_shadow, elevation_shadow_rect, surface_color,
    template_border_width, template_corner_radius,
};
use super::eligibility::draws_border;

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
    let border_width = template_border_width(node);
    let corner_radius = template_corner_radius(node);
    if draws_elevation_shadow(node) {
        commands.push(HostPaintCommand::quad(
            elevation_shadow_rect(rect, node.elevation),
            Some(clip.clone()),
            order - 1,
            Some(PALETTE.shadow),
            None,
            0.0,
            corner_radius,
            MATERIAL_ELEVATION_SHADOW_OPACITY * opacity,
        ));
    }
    if draws_asset_thumbnail_name_area_surface(node, corner_radius) {
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
    if draws_asset_thumbnail_name_area_surface(node, corner_radius) {
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
mod tests {
    use super::*;

    #[test]
    fn asset_thumbnail_name_area_surface_squares_top_edge_over_bottom_rounded_base() {
        let mut node = TemplatePaneNodeData {
            role: "Panel".into(),
            surface_variant: ASSET_THUMBNAIL_NAME_AREA_SURFACE.into(),
            corner_radius: 4.0,
            selected: true,
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 96.0,
            height: 42.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 128.0,
            height: 96.0,
        };
        let mut commands = Vec::new();

        push_surface_commands(&mut commands, &node, &rect, &clip, 7, 1.0);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].frame, rect);
        assert_eq!(commands[0].z_index, 7);
        assert_eq!(commands[0].corner_radius, 4.0);
        assert_eq!(commands[1].frame.x, rect.x);
        assert_eq!(commands[1].frame.y, rect.y);
        assert_eq!(commands[1].frame.width, rect.width);
        assert_eq!(commands[1].frame.height, 4.0);
        assert_eq!(commands[1].z_index, 7);
        assert_eq!(commands[1].background_color, commands[0].background_color);
        assert_eq!(commands[1].border_color, None);
        assert_eq!(commands[1].border_width, 0.0);
        assert_eq!(commands[1].corner_radius, 0.0);

        node.surface_variant = "panel".into();
        commands.clear();
        push_surface_commands(&mut commands, &node, &rect, &clip, 7, 1.0);

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].corner_radius, 4.0);
    }

    #[test]
    fn asset_thumbnail_name_area_state_layer_keeps_square_top_edge() {
        let node = TemplatePaneNodeData {
            role: "Panel".into(),
            surface_variant: ASSET_THUMBNAIL_NAME_AREA_SURFACE.into(),
            corner_radius: 4.0,
            state_layer_enabled: true,
            hovered: true,
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 96.0,
            height: 42.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 128.0,
            height: 96.0,
        };
        let mut commands = Vec::new();

        push_surface_commands(&mut commands, &node, &rect, &clip, 7, 1.0);

        assert_eq!(commands.len(), 4);
        assert_eq!(commands[2].frame, rect);
        assert_eq!(commands[2].z_index, 8);
        assert_eq!(commands[2].corner_radius, 4.0);
        assert_eq!(commands[3].frame.x, rect.x);
        assert_eq!(commands[3].frame.y, rect.y);
        assert_eq!(commands[3].frame.width, rect.width);
        assert_eq!(commands[3].frame.height, 4.0);
        assert_eq!(commands[3].z_index, 8);
        assert_eq!(commands[3].background_color, commands[2].background_color);
        assert_eq!(commands[3].border_color, None);
        assert_eq!(commands[3].border_width, 0.0);
        assert_eq!(commands[3].corner_radius, 0.0);
        assert_eq!(commands[3].opacity, commands[2].opacity);
    }
}
