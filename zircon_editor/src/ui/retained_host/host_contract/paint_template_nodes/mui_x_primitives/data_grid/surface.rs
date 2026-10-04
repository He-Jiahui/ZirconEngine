use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;

pub(super) fn push_data_grid_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    radius: f32,
    opacity: f32,
) {
    let palette = current_host_palette();
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        data_grid_surface_color_from_host(node, palette),
        0.0,
        radius,
        opacity,
    );
}

pub(super) fn push_data_grid_header(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    radius: f32,
    header_height: f32,
    opacity: f32,
) {
    let palette = current_host_palette();
    super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_height,
        },
        clip,
        order,
        data_grid_header_color_from_host(palette),
        0.0,
        radius,
        opacity,
    );
}

fn data_grid_surface_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    super::super::node_background(node).unwrap_or(palette.surface_inset)
}

fn data_grid_header_color_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.surface_hover
}

#[cfg(test)]
#[path = "tests/surface.rs"]
mod tests;
