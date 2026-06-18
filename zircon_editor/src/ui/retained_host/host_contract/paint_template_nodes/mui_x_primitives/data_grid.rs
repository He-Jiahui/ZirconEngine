use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;

const MUI_X_HEADER_HEIGHT_FRACTION: f32 = 0.32;
const MUI_X_ROW_HEIGHT_FRACTION: f32 = 0.22;

pub(super) fn is_data_grid(component_role: &str, role: &str) -> bool {
    super::matches_any_role(component_role, role, &["mui-x-data-grid", "DataGrid"])
}

pub(super) fn push_data_grid(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::node_radius(node).max(4.0);
    super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::node_background(node).unwrap_or(PALETTE.surface_inset),
        0.0,
        radius,
        opacity,
    );
    super::push_quad(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: (rect.height * MUI_X_HEADER_HEIGHT_FRACTION).max(8.0),
        },
        clip,
        order + 1,
        PALETTE.surface_hover,
        0.0,
        radius,
        opacity,
    );

    let first_row_y = rect.y + (rect.height * MUI_X_HEADER_HEIGHT_FRACTION).max(8.0);
    let row_height = (rect.height * MUI_X_ROW_HEIGHT_FRACTION).max(6.0);
    for row in 0..2 {
        let selected = row == 0 && (node.selected || node.checked || node.focused);
        super::push_quad(
            commands,
            FrameRect {
                x: rect.x + 2.0,
                y: first_row_y + row as f32 * row_height,
                width: (rect.width - 4.0).max(1.0),
                height: (row_height - 1.0).max(1.0),
            },
            clip,
            order + 2 + row,
            if selected {
                PALETTE.surface_selected
            } else {
                PALETTE.surface
            },
            0.0,
            2.0,
            opacity,
        );
    }
}
