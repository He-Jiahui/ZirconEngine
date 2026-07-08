use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style_color::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn matches_any_role(
    component_role: &str,
    role: &str,
    expected: &[&str],
) -> bool {
    expected
        .iter()
        .any(|candidate| *candidate == component_role || *candidate == role)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_quad(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    border_width: f32,
    radius: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect,
        Some(clip.clone()),
        order,
        Some(color),
        quad_border_color_from_host(border_width, current_host_palette()),
        border_width,
        radius,
        opacity,
    ));
}

fn quad_border_color_from_host(border_width: f32, palette: HostMaterialPalette) -> Option<[u8; 4]> {
    (border_width > 0.0).then_some(palette.focus_ring)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn node_radius(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn node_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn component_variant_contains(
    node: &TemplatePaneNodeData,
    expected: &str,
) -> bool {
    node.component_variant
        .split_whitespace()
        .any(|part| part == expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_shared_quad_border_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.focus_ring = [10, 11, 12, 255];

        assert_eq!(
            quad_border_color_from_host(1.0, palette),
            Some([10, 11, 12, 255])
        );
    }

    #[test]
    fn mui_x_shared_quad_border_stays_absent_without_width() {
        let mut palette = PALETTE;
        palette.focus_ring = [10, 11, 12, 255];

        assert_eq!(quad_border_color_from_host(0.0, palette), None);
    }
}
