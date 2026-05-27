use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::style::UiStyleColor;

mod charts;

use self::charts::ChartKind;

const MUI_X_HEADER_HEIGHT_FRACTION: f32 = 0.32;
const MUI_X_ROW_HEIGHT_FRACTION: f32 = 0.22;
const MUI_X_TREE_ROW_COUNT: i32 = 3;
const MUI_X_TREE_ROW_HORIZONTAL_INSET: f32 = 4.0;
const MUI_X_TREE_ROW_INDENT_STEP: f32 = 6.0;
const MUI_X_PICKER_FIELD_HEIGHT_FRACTION: f32 = 0.35;
const MUI_X_PICKER_INSET: f32 = 4.0;
const MUI_X_PICKER_SECONDARY: [u8; 4] = [156, 39, 176, 255];
const MUI_X_CHAT_INSET: f32 = 6.0;
const MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION: f32 = 0.24;
const MUI_X_CHAT_STREAMING_HEIGHT: f32 = 3.0;

pub(super) fn push_mui_x_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match mui_x_kind(node) {
        Some(MuiXKind::TreeView) => push_tree_view(commands, node, rect, clip, order, opacity),
        Some(MuiXKind::DataGrid) => push_data_grid(commands, node, rect, clip, order, opacity),
        Some(MuiXKind::DateTimePickers) => {
            push_date_time_picker(commands, node, rect, clip, order, opacity)
        }
        Some(MuiXKind::Chart(kind)) => {
            charts::push_chart(commands, node, rect, clip, order, opacity, kind)
        }
        Some(MuiXKind::AgentChat) => push_agent_chat(commands, node, rect, clip, order, opacity),
        Some(MuiXKind::ChatComposer) => {
            push_chat_composer(commands, node, rect, clip, order, opacity)
        }
        None => return false,
    }
    true
}

enum MuiXKind {
    TreeView,
    DataGrid,
    DateTimePickers,
    Chart(ChartKind),
    AgentChat,
    ChatComposer,
}

fn mui_x_kind(node: &TemplatePaneNodeData) -> Option<MuiXKind> {
    let component_role = node.component_role.as_str();
    let role = node.role.as_str();
    if matches_any_role(
        component_role,
        role,
        &["mui-x-tree-view", "MaterialTreeView", "TreeView"],
    ) {
        Some(MuiXKind::TreeView)
    } else if matches_any_role(component_role, role, &["mui-x-data-grid", "DataGrid"]) {
        Some(MuiXKind::DataGrid)
    } else if matches_any_role(
        component_role,
        role,
        &[
            "mui-x-date-time-pickers",
            "DateTimePickers",
            "DatePicker",
            "TimePicker",
        ],
    ) {
        Some(MuiXKind::DateTimePickers)
    } else if let Some(kind) = charts::chart_kind(component_role, role) {
        Some(MuiXKind::Chart(kind))
    } else if matches_any_role(component_role, role, &["mui-x-agent-chat", "AgentChat"]) {
        Some(MuiXKind::AgentChat)
    } else if matches_any_role(
        component_role,
        role,
        &["mui-x-chat-composer", "ChatComposer"],
    ) {
        Some(MuiXKind::ChatComposer)
    } else {
        None
    }
}

fn matches_any_role(component_role: &str, role: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|candidate| *candidate == component_role || *candidate == role)
}

fn push_tree_view(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = node_radius(node).max(4.0);
    push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        node_background(node).unwrap_or_else(|| tree_view_surface_color(node)),
        0.0,
        radius,
        opacity,
    );

    let row_height = ((rect.height - MUI_X_TREE_ROW_HORIZONTAL_INSET * 2.0)
        / MUI_X_TREE_ROW_COUNT as f32)
        .max(6.0);
    for row in 0..MUI_X_TREE_ROW_COUNT {
        let row_y = rect.y + MUI_X_TREE_ROW_HORIZONTAL_INSET + row as f32 * row_height;
        let row_indent = row as f32 * MUI_X_TREE_ROW_INDENT_STEP;
        let row_rect = FrameRect {
            x: rect.x + MUI_X_TREE_ROW_HORIZONTAL_INSET + row_indent,
            y: row_y,
            width: (rect.width - MUI_X_TREE_ROW_HORIZONTAL_INSET * 2.0 - row_indent).max(1.0),
            height: (row_height - 1.0).max(1.0),
        };
        push_quad(
            commands,
            row_rect.clone(),
            clip,
            order + 1 + row,
            tree_view_row_color(node, row),
            0.0,
            4.0,
            opacity,
        );
        let marker_size = (row_rect.height * 0.45).max(3.0).min(6.0);
        push_quad(
            commands,
            FrameRect {
                x: row_rect.x + 3.0,
                y: row_rect.y + (row_rect.height - marker_size) * 0.5,
                width: marker_size,
                height: marker_size,
            },
            clip,
            order + 5 + row,
            if row == 0 && (node.expanded || node.popup_open) {
                PALETTE.success
            } else {
                PALETTE.border
            },
            0.0,
            marker_size * 0.5,
            opacity,
        );
    }
}

fn push_data_grid(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = node_radius(node).max(4.0);
    push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        node_background(node).unwrap_or(PALETTE.surface_inset),
        0.0,
        radius,
        opacity,
    );
    push_quad(
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
        push_quad(
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

fn push_date_time_picker(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = node_radius(node).max(4.0);
    push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        node_background(node).unwrap_or(PALETTE.surface_inset),
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
    push_quad(
        commands,
        field.clone(),
        clip,
        order + 1,
        PALETTE.surface_inset,
        1.0,
        4.0,
        opacity,
    );
    push_quad(
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

    if node.popup_open || component_variant_contains(node, "desktop") || node.selected {
        let layout = FrameRect {
            x: rect.x + MUI_X_PICKER_INSET,
            y: field.y + field.height + MUI_X_PICKER_INSET,
            width: (rect.width - MUI_X_PICKER_INSET * 2.0).max(1.0),
            height: (rect.y + rect.height - field.y - field.height - MUI_X_PICKER_INSET * 2.0)
                .max(8.0),
        };
        push_quad(
            commands,
            layout.clone(),
            clip,
            order + 3,
            PALETTE.surface,
            0.0,
            4.0,
            opacity,
        );
        push_quad(
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
        push_quad(
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

fn push_agent_chat(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = node_radius(node).max(8.0);
    push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        node_background(node).unwrap_or_else(|| chat_surface_color(node)),
        0.0,
        radius,
        opacity,
    );

    let bubble_height = (rect.height * MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION).max(8.0);
    push_quad(
        commands,
        FrameRect {
            x: rect.x + MUI_X_CHAT_INSET,
            y: rect.y + MUI_X_CHAT_INSET,
            width: rect.width * 0.58,
            height: bubble_height,
        },
        clip,
        order + 1,
        PALETTE.surface,
        0.0,
        5.0,
        opacity,
    );
    push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.36,
            y: rect.y + MUI_X_CHAT_INSET + bubble_height + 3.0,
            width: (rect.width * 0.58 - MUI_X_CHAT_INSET).max(1.0),
            height: bubble_height,
        },
        clip,
        order + 2,
        PALETTE.surface_selected,
        0.0,
        5.0,
        opacity,
    );

    if node.component_variant.as_str().contains("streaming") || node.popup_open {
        push_quad(
            commands,
            FrameRect {
                x: rect.x + MUI_X_CHAT_INSET,
                y: rect.y + rect.height - MUI_X_CHAT_INSET,
                width: (rect.width * 0.42).max(1.0),
                height: MUI_X_CHAT_STREAMING_HEIGHT,
            },
            clip,
            order + 3,
            PALETTE.accent,
            0.0,
            MUI_X_CHAT_STREAMING_HEIGHT * 0.5,
            opacity,
        );
    }
}

fn push_chat_composer(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = node_radius(node).max(rect.height * 0.5);
    push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        node_background(node).unwrap_or(PALETTE.surface_inset),
        1.0,
        radius,
        opacity,
    );
    push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width - rect.height + 4.0,
            y: rect.y + 4.0,
            width: (rect.height - 8.0).max(1.0),
            height: (rect.height - 8.0).max(1.0),
        },
        clip,
        order + 1,
        PALETTE.accent,
        0.0,
        rect.height,
        opacity,
    );
}

fn push_quad(
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
        (border_width > 0.0).then_some(PALETTE.focus_ring),
        border_width,
        radius,
        opacity,
    ));
}

fn chart_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.component_variant.as_str().contains("loading") {
        PALETTE.warning_container
    } else {
        PALETTE.surface_inset
    }
}

fn chat_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error_container
    } else if node.component_variant.as_str().contains("streaming") {
        PALETTE.info_container
    } else {
        PALETTE.surface_inset
    }
}

fn node_radius(node: &TemplatePaneNodeData) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}

fn node_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

fn resolved_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Inherit => None,
        UiStyleColor::Role(role) => match role.as_str() {
            "primary" | "accent" | "material.primary" | "material_color_primary" => {
                Some(PALETTE.accent)
            }
            "surface" | "material.surface" => Some(PALETTE.surface),
            "surface_inset" | "material.surface_inset" => Some(PALETTE.surface_inset),
            "surface_hover" | "material.surface_hover" => Some(PALETTE.surface_hover),
            "surface_selected" | "material.surface_selected" => Some(PALETTE.surface_selected),
            "warning" | "material.warning" => Some(PALETTE.warning),
            "error" | "danger" | "material.error" => Some(PALETTE.error),
            "success" | "material.success" => Some(PALETTE.success),
            "info" | "material.info" => Some(PALETTE.info),
            _ => None,
        },
    }
}

fn tree_view_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.selected || node.checked || component_variant_contains(node, "multi") {
        PALETTE.success_container
    } else {
        PALETTE.surface_inset
    }
}

fn tree_view_row_color(node: &TemplatePaneNodeData, row: i32) -> [u8; 4] {
    if row == 0 && (node.selected || node.checked) {
        PALETTE.surface_selected
    } else if row == 1 && (node.expanded || node.popup_open || node.focused) {
        PALETTE.surface_hover
    } else {
        PALETTE.surface
    }
}

fn component_variant_contains(node: &TemplatePaneNodeData, expected: &str) -> bool {
    node.component_variant
        .split_whitespace()
        .any(|part| part == expected)
}
