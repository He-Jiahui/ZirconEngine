use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::style_selector::WorkbenchButtonKind;
use super::super::geometry::button_radius;
use super::super::style::button_style;

pub(super) struct ButtonSurfaceCommandStyle {
    pub fill: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn button_surface_command_style(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    kind: WorkbenchButtonKind,
) -> ButtonSurfaceCommandStyle {
    let style = button_style(node, kind);
    ButtonSurfaceCommandStyle {
        fill: style.surface,
        border: style.border,
        border_width: style.border_width,
        radius: button_radius(node, rect),
    }
}
