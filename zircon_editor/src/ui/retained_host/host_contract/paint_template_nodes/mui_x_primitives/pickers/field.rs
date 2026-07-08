use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;
use super::super::{node_background, node_radius, push_quad};
use super::geometry::{picker_field_frame, picker_field_icon_frame};
use super::metrics::{PICKER_FIELD_RADIUS, PICKER_ROOT_BORDER_WIDTH};

type PickerFieldColors = [[u8; 4]; 3];

pub(super) fn push_picker_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> FrameRect {
    let [root_surface, field_surface, icon_surface] =
        picker_field_colors_from_host(node, current_host_palette());
    push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        root_surface,
        PICKER_ROOT_BORDER_WIDTH,
        node_radius(node).max(PICKER_FIELD_RADIUS),
        opacity,
    );

    let field = picker_field_frame(rect);
    push_quad(
        commands,
        field.clone(),
        clip,
        order + 1,
        field_surface,
        PICKER_ROOT_BORDER_WIDTH,
        PICKER_FIELD_RADIUS,
        opacity,
    );
    push_quad(
        commands,
        picker_field_icon_frame(&field),
        clip,
        order + 2,
        icon_surface,
        0.0,
        PICKER_FIELD_RADIUS,
        opacity,
    );
    field
}

fn picker_field_colors_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> PickerFieldColors {
    [
        node_background(node).unwrap_or(palette.surface_inset),
        palette.surface_inset,
        palette.accent_soft,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_picker_field_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_inset = [10, 11, 12, 255];
        palette.accent_soft = [20, 21, 22, 255];

        assert_eq!(
            picker_field_colors_from_host(&TemplatePaneNodeData::default(), palette),
            [[10, 11, 12, 255], [10, 11, 12, 255], [20, 21, 22, 255],]
        );
    }
}
