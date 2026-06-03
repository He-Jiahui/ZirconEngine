use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_node_labels::template_node_label;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const STATUS_FONT_SIZE: f32 = 12.0;
const STATUS_ITEM_ICON_LEFT: f32 = 24.0;
const STATUS_ITEM_TEXT_GAP: f32 = 9.0;
const STATUS_ITEM_ICON_SIZE: f32 = 14.0;
const STATUS_READY_DOT_SIZE: f32 = 10.0;
const STATUS_CHIP_TEXT_LEFT: f32 = 12.0;
const STATUS_CHIP_RIGHT_RESERVE: f32 = 24.0;
const STATUS_CHIP_RADIUS: f32 = 6.0;
const STATUS_ICON_GLYPH_SIZE: f32 = 16.0;
const STATUS_ICON_BUTTON_RADIUS: f32 = 5.0;
const STATUS_ICON_COLOR: [u8; 4] = [149, 164, 172, 255];
const STATUS_ICON_MUTED: [u8; 4] = [105, 121, 130, 255];
const STATUS_RIGHT_BORDER: [u8; 4] = [36, 44, 50, 255];
const STATUS_NO_ERRORS_FILL: [u8; 4] = [88, 184, 102, 255];
const STATUS_MARK_ON_LIGHT: [u8; 4] = [8, 18, 18, 255];

pub(super) fn push_status_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match status_control_kind(node) {
        Some(StatusControlKind::Signal(kind)) => {
            push_status_signal_item(commands, node, rect, clip, order, kind, opacity);
            true
        }
        Some(StatusControlKind::Chip) => {
            push_status_chip(commands, node, rect, clip, order, opacity);
            true
        }
        Some(StatusControlKind::Icon(kind)) => {
            push_status_icon_button(commands, node, rect, clip, order, kind, opacity);
            true
        }
        None => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatusControlKind {
    Signal(StatusSignalKind),
    Chip,
    Icon(StatusIconKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatusSignalKind {
    Ready,
    Success,
    Warning,
    Info,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatusIconKind {
    Snap,
    World,
    Target,
}

fn status_control_kind(node: &TemplatePaneNodeData) -> Option<StatusControlKind> {
    match node.control_id.as_str() {
        "WorkbenchStatusReady" => Some(StatusControlKind::Signal(StatusSignalKind::Ready)),
        "WorkbenchStatusErrors" => Some(StatusControlKind::Signal(StatusSignalKind::Success)),
        "WorkbenchStatusWarnings" => Some(StatusControlKind::Signal(StatusSignalKind::Warning)),
        "WorkbenchStatusMessages" => Some(StatusControlKind::Signal(StatusSignalKind::Info)),
        "WorkbenchStatusGrid" | "WorkbenchStatusSnap" | "WorkbenchStatusZoom" => {
            Some(StatusControlKind::Chip)
        }
        "WorkbenchStatusSnapToggle" => Some(StatusControlKind::Icon(StatusIconKind::Snap)),
        "WorkbenchStatusWorld" => Some(StatusControlKind::Icon(StatusIconKind::World)),
        "WorkbenchStatusTarget" => Some(StatusControlKind::Icon(StatusIconKind::Target)),
        _ => None,
    }
}

fn push_status_signal_item(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusSignalKind,
    opacity: f32,
) {
    let icon = status_signal_icon_rect(node, rect, kind);
    let icon_paint = status_signal_icon_paint_rect(node, &icon, kind);
    push_status_signal_icon(commands, node, &icon_paint, clip, order, kind, opacity);
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let line_height = STATUS_FONT_SIZE * 1.2;
    let text_gap = status_signal_text_gap(node);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: icon.x + icon.width + text_gap,
            y: rect.y + node.layout_offset_y + (rect.height - line_height).max(0.0) * 0.5,
            width: (rect.x + rect.width - icon.x - icon.width - text_gap).max(1.0),
            height: line_height,
        },
        Some(clip.clone()),
        order + 2,
        label,
        status_signal_text_color(node, kind),
        STATUS_FONT_SIZE,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_status_chip(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let rect = status_control_offset_rect(node, rect);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(status_chip_background(node)),
        Some(status_chip_border(node)),
        1.0,
        STATUS_CHIP_RADIUS,
        opacity,
    ));

    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        let line_height = STATUS_FONT_SIZE * 1.2;
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: rect.x + STATUS_CHIP_TEXT_LEFT,
                y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
                width: (rect.width - STATUS_CHIP_TEXT_LEFT - STATUS_CHIP_RIGHT_RESERVE).max(1.0),
                height: line_height,
            },
            Some(clip.clone()),
            order + 2,
            label,
            status_chip_text_color(node),
            STATUS_FONT_SIZE,
            line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    let chevron = FrameRect {
        x: rect.x + rect.width - 18.0,
        y: rect.y + (rect.height - 12.0).max(0.0) * 0.5,
        width: 12.0,
        height: 12.0,
    };
    push_down_chevron(
        commands,
        &chevron,
        clip,
        order + 3,
        status_chip_text_color(node),
        opacity,
    );
}

fn status_control_offset_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

fn push_status_icon_button(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusIconKind,
    opacity: f32,
) {
    let rect = status_control_offset_rect(node, rect);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(status_icon_button_background(node)),
        Some(status_icon_button_border(node)),
        1.0,
        STATUS_ICON_BUTTON_RADIUS,
        opacity,
    ));
    let glyph = centered_rect(&rect, STATUS_ICON_GLYPH_SIZE);
    push_status_icon_glyph(
        commands,
        &glyph,
        clip,
        order + 2,
        kind,
        status_icon_glyph_color(node),
        opacity,
    );
}

fn status_signal_icon_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    kind: StatusSignalKind,
) -> FrameRect {
    let size = status_signal_icon_size(node, kind);
    FrameRect {
        x: rect.x + STATUS_ITEM_ICON_LEFT + node.layout_offset_x,
        y: rect.y
            + node.layout_offset_y
            + (rect.height - size).max(0.0) * 0.5
            + node.layout_content_offset_y,
        width: size,
        height: size,
    }
}

fn status_signal_icon_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    kind: StatusSignalKind,
) -> FrameRect {
    let size = status_signal_visual_icon_size(node, kind)
        .min(rect.width.min(rect.height).max(1.0))
        .max(1.0);
    centered_rect(rect, size)
}

fn push_status_signal_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusSignalKind,
    opacity: f32,
) {
    match kind {
        StatusSignalKind::Ready => commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(status_signal_icon_fill(node, kind)),
            None,
            0.0,
            rect.height * 0.5,
            opacity,
        )),
        StatusSignalKind::Success => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(status_signal_icon_fill(node, kind)),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_check_mark(
                commands,
                rect,
                clip,
                order + 1,
                status_signal_mark_color(node),
                opacity,
            );
        }
        StatusSignalKind::Warning => {
            push_warning_triangle(
                commands,
                rect,
                clip,
                order,
                status_signal_icon_fill(node, kind),
                status_signal_mark_color(node),
                status_signal_mark_width(node),
                opacity,
            );
        }
        StatusSignalKind::Info => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(status_signal_icon_fill(node, kind)),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_segments(
                commands,
                clip,
                order + 1,
                status_signal_mark_color(node),
                opacity,
                &[
                    local_rect_scaled(rect, 6.0, 3.0, 2.0, 2.0, STATUS_ITEM_ICON_SIZE),
                    local_rect_scaled(rect, 6.0, 6.0, 2.0, 5.0, STATUS_ITEM_ICON_SIZE),
                ],
            );
        }
    }
}

fn push_status_icon_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusIconKind,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        StatusIconKind::Snap => push_snap_icon(commands, rect, clip, order, color, opacity),
        StatusIconKind::World => push_world_icon(commands, rect, clip, order, color, opacity),
        StatusIconKind::Target => push_target_icon(commands, rect, clip, order, color, opacity),
    }
}

fn push_snap_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 4.0, 3.0, 8.0),
            local_rect(rect, 10.0, 4.0, 3.0, 8.0),
            local_rect(rect, 3.0, 11.0, 10.0, 3.0),
            local_rect(rect, 4.0, 2.0, 2.0, 3.0),
            local_rect(rect, 10.0, 2.0, 2.0, 3.0),
        ],
    );
}

fn push_world_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        1.0,
        rect.height * 0.5,
        opacity,
    ));
    push_segments(
        commands,
        clip,
        order + 1,
        color,
        opacity,
        &[
            local_rect(rect, 7.0, 2.0, 2.0, 12.0),
            local_rect(rect, 3.0, 7.0, 10.0, 2.0),
            local_rect(rect, 4.0, 4.0, 8.0, 1.0),
            local_rect(rect, 4.0, 11.0, 8.0, 1.0),
        ],
    );
}

fn push_target_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        1.0,
        rect.height * 0.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        centered_rect(rect, 4.0),
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        2.0,
        opacity,
    ));
    push_segments(
        commands,
        clip,
        order + 1,
        color,
        opacity,
        &[
            local_rect(rect, 7.0, 0.0, 2.0, 4.0),
            local_rect(rect, 7.0, 12.0, 2.0, 4.0),
            local_rect(rect, 0.0, 7.0, 4.0, 2.0),
            local_rect(rect, 12.0, 7.0, 4.0, 2.0),
        ],
    );
}

fn push_warning_triangle(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    mark_color: [u8; 4],
    mark_width: f32,
    opacity: f32,
) {
    let center_x = rect.x + rect.width * 0.5;
    let scale_x = rect.width / STATUS_ITEM_ICON_SIZE;
    let scale_y = rect.height / STATUS_ITEM_ICON_SIZE;
    for (row, width) in [2.0, 4.0, 6.0, 8.0, 10.0, 12.0].into_iter().enumerate() {
        let width = width * scale_x;
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: center_x - width * 0.5,
                y: rect.y + (2.0 + row as f32 * 1.7) * scale_y,
                width,
                height: 2.0 * scale_y,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
    push_segments(
        commands,
        clip,
        order + 1,
        mark_color,
        opacity,
        &warning_mark_segments(rect, mark_width),
    );
}

fn warning_mark_segments(rect: &FrameRect, mark_width: f32) -> [FrameRect; 2] {
    let mark_width = normalized_status_mark_width(mark_width);
    let x = 7.0 - mark_width * 0.5;
    [
        local_rect_scaled(rect, x, 6.0, mark_width, 4.0, STATUS_ITEM_ICON_SIZE),
        local_rect_scaled(rect, x, 11.0, mark_width, mark_width, STATUS_ITEM_ICON_SIZE),
    ]
}

fn push_check_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect_scaled(rect, 3.0, 7.0, 3.0, 2.0, STATUS_ITEM_ICON_SIZE),
            local_rect_scaled(rect, 5.0, 9.0, 3.0, 2.0, STATUS_ITEM_ICON_SIZE),
            local_rect_scaled(rect, 8.0, 4.0, 3.0, 7.0, STATUS_ITEM_ICON_SIZE),
        ],
    );
}

fn push_down_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 4.0, 2.0, 2.0),
            local_rect(rect, 5.0, 6.0, 2.0, 2.0),
            local_rect(rect, 7.0, 4.0, 2.0, 2.0),
        ],
    );
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[FrameRect],
) {
    for segment in segments {
        commands.push(HostPaintCommand::quad(
            segment.clone(),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn centered_rect(rect: &FrameRect, size: f32) -> FrameRect {
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size.min(rect.width.max(1.0)).max(1.0),
        height: size.min(rect.height.max(1.0)).max(1.0),
    }
}

fn local_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
}

fn local_rect_scaled(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    base_size: f32,
) -> FrameRect {
    let scale_x = origin.width / base_size;
    let scale_y = origin.height / base_size;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: width * scale_x,
        height: height * scale_y,
    }
}

fn status_signal_icon_size(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> f32 {
    if node.value_number > 0.0 {
        return node.value_number;
    }
    match kind {
        StatusSignalKind::Ready => STATUS_READY_DOT_SIZE,
        StatusSignalKind::Success | StatusSignalKind::Warning | StatusSignalKind::Info => {
            STATUS_ITEM_ICON_SIZE
        }
    }
}

fn status_signal_visual_icon_size(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> f32 {
    if node.layout_icon_size.is_finite() && node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        status_signal_icon_size(node, kind)
    }
}

fn status_signal_text_gap(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else {
        STATUS_ITEM_TEXT_GAP
    }
}

fn status_signal_icon_fill(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> [u8; 4] {
    if let Some(color) = declared_color(node.label_color) {
        return color;
    }
    match kind {
        StatusSignalKind::Ready => PALETTE.success,
        StatusSignalKind::Success => STATUS_NO_ERRORS_FILL,
        StatusSignalKind::Warning => PALETTE.warning,
        StatusSignalKind::Info => PALETTE.info,
    }
}

fn status_signal_text_color(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    if let Some(color) = declared_color(node.value_color) {
        return color;
    }
    match kind {
        StatusSignalKind::Ready => PALETTE.text,
        StatusSignalKind::Success | StatusSignalKind::Warning | StatusSignalKind::Info => {
            PALETTE.text_muted
        }
    }
}

fn status_signal_mark_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    declared_color(node.icon_color).unwrap_or(STATUS_MARK_ON_LIGHT)
}

fn status_signal_mark_width(node: &TemplatePaneNodeData) -> f32 {
    normalized_status_mark_width(node.icon_stroke_width)
}

fn normalized_status_mark_width(width: f32) -> f32 {
    if width.is_finite() && width > 0.0 {
        width
    } else {
        2.0
    }
}

fn declared_color(color: crate::ui::retained_host::primitives::Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

fn status_chip_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.surface_disabled
    } else if node.pressed {
        PALETTE.surface_pressed
    } else if node.hovered || node.focused {
        PALETTE.surface_hover
    } else {
        PALETTE.surface_inset
    }
}

fn status_chip_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.border_disabled
    } else if node.focused || node.pressed {
        PALETTE.focus_ring
    } else {
        STATUS_RIGHT_BORDER
    }
}

fn status_chip_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else {
        PALETTE.text_muted
    }
}

fn status_icon_button_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.surface_disabled
    } else if node.checked || node.selected {
        PALETTE.surface_selected
    } else if node.pressed {
        PALETTE.surface_pressed
    } else if node.hovered || node.focused {
        PALETTE.surface_hover
    } else {
        PALETTE.surface_inset
    }
}

fn status_icon_button_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.border_disabled
    } else if node.checked || node.selected || node.focused || node.pressed {
        PALETTE.focus_ring
    } else {
        STATUS_RIGHT_BORDER
    }
}

fn status_icon_glyph_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if node.checked || node.selected || node.focused || node.pressed {
        PALETTE.focus_ring
    } else if node.hovered {
        STATUS_ICON_COLOR
    } else {
        STATUS_ICON_MUTED
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn status_control_kind_matches_workbench_status_ids() {
        assert_eq!(
            status_control_kind(&status_node("WorkbenchStatusReady", "Ready", 96.0, 46.0)),
            Some(StatusControlKind::Signal(StatusSignalKind::Ready))
        );
        assert_eq!(
            status_control_kind(&status_node(
                "WorkbenchStatusGrid",
                "Grid: 10 cm",
                112.0,
                30.0
            )),
            Some(StatusControlKind::Chip)
        );
        assert_eq!(
            status_control_kind(&status_node("WorkbenchStatusTarget", "", 34.0, 30.0)),
            Some(StatusControlKind::Icon(StatusIconKind::Target))
        );
        assert_eq!(
            status_control_kind(&status_node("WorkbenchStatusFill", "", 80.0, 46.0)),
            None
        );
    }

    #[test]
    fn ready_status_item_paints_dot_and_text_without_chip_surface() {
        let bytes = paint_template_nodes_for_test(
            140,
            46,
            model_rc(vec![status_node(
                "WorkbenchStatusReady",
                "Ready",
                96.0,
                46.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 140, 29, 23), PALETTE.success);
        assert_eq!(pixel_at(&bytes, 140, 90, 4), [0, 0, 0, 255]);
        assert!(changed_pixel_count(&bytes, 140, 42, 14, 40, 18) > 0);
    }

    #[test]
    fn ready_status_item_uses_declared_dot_text_and_gap_style() {
        let mut node = status_node("WorkbenchStatusReady", "Ready", 96.0, 46.0);
        node.layout_offset_x = 4.0;
        node.layout_offset_y = -1.0;
        node.layout_content_offset_x = 8.0;
        node.value_number = 9.0;
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(143, 154, 160);
        node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(78, 170, 95);

        let icon = status_signal_icon_rect(
            &node,
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 96.0,
                height: 46.0,
            },
            StatusSignalKind::Ready,
        );

        assert!((icon.x - 28.0).abs() < 0.001);
        assert!((icon.y - 17.5).abs() < 0.001);
        assert!((icon.width - 9.0).abs() < 0.001);
        assert!((status_signal_text_gap(&node) - 8.0).abs() < 0.001);
        assert_eq!(
            status_signal_text_color(&node, StatusSignalKind::Ready),
            [143, 154, 160, 255]
        );
        assert_eq!(
            status_signal_icon_fill(&node, StatusSignalKind::Ready),
            [78, 170, 95, 255]
        );
    }

    #[test]
    fn errors_status_item_uses_audited_success_icon_fill() {
        let bytes = paint_template_nodes_for_test(
            140,
            46,
            model_rc(vec![status_node(
                "WorkbenchStatusErrors",
                "No Errors",
                116.0,
                46.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 140, 31, 23), STATUS_NO_ERRORS_FILL);
        assert!(changed_pixel_count(&bytes, 140, 46, 14, 58, 18) > 0);
    }

    #[test]
    fn errors_status_item_uses_declared_success_mark_color() {
        let mut node = status_node("WorkbenchStatusErrors", "No Errors", 116.0, 46.0);
        node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(17, 32, 24);

        assert_eq!(status_signal_mark_color(&node), [17, 32, 24, 255]);
    }

    #[test]
    fn errors_status_item_uses_declared_visual_icon_size_without_moving_text_slot() {
        let mut node = status_node("WorkbenchStatusErrors", "No Errors", 116.0, 46.0);
        node.layout_icon_size = 12.04;

        let layout = status_signal_icon_rect(
            &node,
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 116.0,
                height: 46.0,
            },
            StatusSignalKind::Success,
        );
        let paint = status_signal_icon_paint_rect(&node, &layout, StatusSignalKind::Success);

        assert!((layout.x - 24.0).abs() < 0.001);
        assert!((layout.width - 14.0).abs() < 0.001);
        assert!((paint.x - 24.98).abs() < 0.001);
        assert!((paint.width - 12.04).abs() < 0.001);
    }

    #[test]
    fn warning_status_item_uses_declared_icon_text_and_gap_style() {
        let mut node = status_node("WorkbenchStatusWarnings", "2 Warnings", 120.0, 46.0);
        node.layout_offset_x = 5.5;
        node.layout_offset_y = -2.0;
        node.layout_content_offset_x = 6.45;
        node.layout_content_offset_y = -2.0;
        node.value_number = 21.0;
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(135, 146, 153);
        node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(242, 195, 86);
        node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(17, 24, 26);
        node.icon_stroke_width = 1.45;

        let icon = status_signal_icon_rect(
            &node,
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 120.0,
                height: 46.0,
            },
            StatusSignalKind::Warning,
        );

        assert!((icon.x - 29.5).abs() < 0.001);
        assert!((icon.y - 8.5).abs() < 0.001);
        assert!((icon.width - 21.0).abs() < 0.001);
        assert!((status_signal_text_gap(&node) - 6.45).abs() < 0.001);
        assert_eq!(
            status_signal_text_color(&node, StatusSignalKind::Warning),
            [135, 146, 153, 255]
        );
        assert_eq!(
            status_signal_icon_fill(&node, StatusSignalKind::Warning),
            [242, 195, 86, 255]
        );
        assert_eq!(status_signal_mark_color(&node), [17, 24, 26, 255]);
        assert!((status_signal_mark_width(&node) - 1.45).abs() < 0.001);
        let mark_segments = warning_mark_segments(&icon, status_signal_mark_width(&node));
        assert!((mark_segments[0].x - 38.9125).abs() < 0.001);
        assert!((mark_segments[0].width - 2.175).abs() < 0.001);
        assert!((mark_segments[1].height - 2.175).abs() < 0.001);
    }

    #[test]
    fn messages_status_item_uses_declared_icon_text_and_offset_style() {
        let mut node = status_node("WorkbenchStatusMessages", "0 Messages", 130.0, 46.0);
        node.layout_offset_x = -6.0;
        node.layout_offset_y = -2.0;
        node.layout_content_offset_y = 2.0;
        node.value_number = 18.0;
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(151, 163, 169);
        node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(76, 154, 232);

        let icon = status_signal_icon_rect(
            &node,
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 130.0,
                height: 46.0,
            },
            StatusSignalKind::Info,
        );

        assert!((icon.x - 18.0).abs() < 0.001);
        assert!((icon.y - 14.0).abs() < 0.001);
        assert!((icon.width - 18.0).abs() < 0.001);
        assert_eq!(
            status_signal_text_color(&node, StatusSignalKind::Info),
            [151, 163, 169, 255]
        );
        assert_eq!(
            status_signal_icon_fill(&node, StatusSignalKind::Info),
            [76, 154, 232, 255]
        );
    }

    #[test]
    fn status_chip_paints_pill_surface_and_down_chevron() {
        let bytes = paint_template_nodes_for_test(
            140,
            48,
            model_rc(vec![status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm")]),
        );

        assert_ne!(pixel_at(&bytes, 140, 20, 20), [0, 0, 0, 255]);
        assert_eq!(pixel_at(&bytes, 140, 60, 9), STATUS_RIGHT_BORDER);
        assert!(changed_pixel_count(&bytes, 140, 101, 18, 18, 14) > 0);
    }

    #[test]
    fn status_chip_uses_declared_text_color_and_layout_offset() {
        let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
        node.layout_offset_y = -2.0;
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144);

        let rect = status_control_offset_rect(
            &node,
            &FrameRect {
                x: node.frame.x,
                y: node.frame.y,
                width: node.frame.width,
                height: node.frame.height,
            },
        );

        assert!((rect.y - 7.0).abs() < 0.001);
        assert_eq!(status_chip_text_color(&node), [125, 137, 144, 255]);
    }

    #[test]
    fn status_icon_button_paints_target_glyph() {
        let bytes = paint_template_nodes_for_test(
            48,
            42,
            model_rc(vec![status_icon_node("WorkbenchStatusTarget")]),
        );

        assert_ne!(pixel_at(&bytes, 48, 8, 8), [0, 0, 0, 255]);
        assert_eq!(pixel_at(&bytes, 48, 24, 6), STATUS_RIGHT_BORDER);
        assert!(changed_pixel_count(&bytes, 48, 14, 11, 20, 20) > 0);
    }

    #[test]
    fn status_icon_button_uses_declared_layout_offset() {
        let mut node = status_icon_node("WorkbenchStatusTarget");
        node.layout_offset_y = -2.0;

        let rect = status_control_offset_rect(
            &node,
            &FrameRect {
                x: 6.0,
                y: 6.0,
                width: 34.0,
                height: 30.0,
            },
        );

        assert!((rect.y - 4.0).abs() < 0.001);
    }

    fn status_node(control_id: &str, text: &str, width: f32, height: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Label".into(),
            text: text.into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn status_chip_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 9.0,
                width: 112.0,
                height: 30.0,
            },
            ..status_node(control_id, text, 112.0, 30.0)
        }
    }

    fn status_icon_node(control_id: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "IconButton".into(),
            frame: TemplateNodeFrameData {
                x: 6.0,
                y: 6.0,
                width: 34.0,
                height: 30.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn changed_pixel_count(
        bytes: &[u8],
        frame_width: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> usize {
        let mut changed = 0;
        for py in y..(y + height) {
            for px in x..(x + width) {
                let index = ((py as usize * frame_width as usize) + px as usize) * 4;
                if bytes[index..index + 4] != [0, 0, 0, 255] {
                    changed += 1;
                }
            }
        }
        changed
    }

    fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y as usize * frame_width as usize) + x as usize) * 4;
        [
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]
    }
}
