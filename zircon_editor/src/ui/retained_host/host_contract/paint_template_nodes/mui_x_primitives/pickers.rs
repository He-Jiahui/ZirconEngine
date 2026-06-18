use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;

const MUI_X_PICKER_FIELD_HEIGHT_FRACTION: f32 = 0.35;
const MUI_X_PICKER_INSET: f32 = 4.0;
const MUI_X_PICKER_SECONDARY: [u8; 4] = [156, 39, 176, 255];

pub(super) fn is_date_time_picker(component_role: &str, role: &str) -> bool {
    super::matches_any_role(
        component_role,
        role,
        &[
            "mui-x-date-time-pickers",
            "DateTimePickers",
            "DatePicker",
            "TimePicker",
        ],
    )
}

pub(super) fn push_date_time_picker(
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
        1.0,
        radius,
        opacity,
    );

    let field_height = (rect.height * MUI_X_PICKER_FIELD_HEIGHT_FRACTION).max(12.0);
    let field = FrameRect {
        x: rect.x + MUI_X_PICKER_INSET,
        y: rect.y + MUI_X_PICKER_INSET,
        width: (rect.width - MUI_X_PICKER_INSET * 2.0).max(1.0),
        height: field_height,
    };
    super::push_quad(
        commands,
        field.clone(),
        clip,
        order + 1,
        PALETTE.surface_inset,
        1.0,
        4.0,
        opacity,
    );
    super::push_quad(
        commands,
        FrameRect {
            x: field.x + field.width - field.height + 3.0,
            y: field.y + 3.0,
            width: (field.height - 6.0).max(1.0),
            height: (field.height - 6.0).max(1.0),
        },
        clip,
        order + 2,
        MUI_X_PICKER_SECONDARY,
        0.0,
        4.0,
        opacity,
    );

    if node.popup_open || super::component_variant_contains(node, "desktop") || node.selected {
        let layout = FrameRect {
            x: rect.x + MUI_X_PICKER_INSET,
            y: field.y + field.height + MUI_X_PICKER_INSET,
            width: (rect.width - MUI_X_PICKER_INSET * 2.0).max(1.0),
            height: (rect.y + rect.height - field.y - field.height - MUI_X_PICKER_INSET * 2.0)
                .max(8.0),
        };
        super::push_quad(
            commands,
            layout.clone(),
            clip,
            order + 3,
            PALETTE.surface,
            0.0,
            4.0,
            opacity,
        );
        super::push_quad(
            commands,
            FrameRect {
                x: layout.x,
                y: layout.y,
                width: layout.width,
                height: 5.0_f32.min(layout.height),
            },
            clip,
            order + 4,
            MUI_X_PICKER_SECONDARY,
            0.0,
            4.0,
            opacity,
        );
        let cell_size = (layout.width / 7.0).min(layout.height - 8.0).max(4.0);
        super::push_quad(
            commands,
            FrameRect {
                x: layout.x + layout.width * 0.5 - cell_size * 0.5,
                y: layout.y + layout.height * 0.58 - cell_size * 0.5,
                width: cell_size,
                height: cell_size,
            },
            clip,
            order + 5,
            MUI_X_PICKER_SECONDARY,
            0.0,
            cell_size * 0.5,
            opacity,
        );
    }
}
