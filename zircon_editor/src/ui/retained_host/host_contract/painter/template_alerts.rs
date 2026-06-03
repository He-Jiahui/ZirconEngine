use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::style_selector::select_workbench_toast_style;
use super::template_node_labels::template_node_label;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const ALERT_FONT_SIZE: f32 = 12.0;
const ALERT_LINE_HEIGHT: f32 = ALERT_FONT_SIZE * 1.2;
const ALERT_RADIUS: f32 = 4.0;
const ALERT_BORDER_WIDTH: f32 = 1.0;
const ALERT_ICON_LEFT: f32 = 10.0;
const ALERT_ICON_SIZE: f32 = 18.0;
const ALERT_TEXT_GAP: f32 = 8.0;
const ALERT_TEXT_RIGHT_INSET: f32 = 10.0;
const ALERT_GLYPH_DARK: [u8; 4] = [8, 18, 18, 255];

const TOAST_FONT_SIZE: f32 = 11.5;
const TOAST_LINE_HEIGHT: f32 = TOAST_FONT_SIZE * 1.25;
const TOAST_RADIUS: f32 = 5.0;
const TOAST_ICON_LEFT: f32 = 12.0;
const TOAST_ICON_SIZE: f32 = 18.0;
const TOAST_TEXT_GAP: f32 = 9.0;
const TOAST_TRAILING_INSET: f32 = 10.0;
const TOAST_CLOSE_SIZE: f32 = 14.0;
const TOAST_ACTION_WIDTH: f32 = 44.0;
const TOAST_ACTION_TEXT: &str = "UNDO";

const INFO_SURFACE: [u8; 4] = [18, 46, 72, 255];
const INFO_BORDER: [u8; 4] = [41, 101, 150, 255];
const SUCCESS_SURFACE: [u8; 4] = [22, 57, 39, 255];
const SUCCESS_BORDER: [u8; 4] = [53, 115, 72, 255];
const WARNING_SURFACE: [u8; 4] = [69, 50, 20, 255];
const WARNING_BORDER: [u8; 4] = [132, 94, 35, 255];
const ERROR_SURFACE: [u8; 4] = [72, 32, 36, 255];
const ERROR_BORDER: [u8; 4] = [133, 61, 58, 255];

pub(super) fn push_alert_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = workbench_alert_kind(node) else {
        return false;
    };
    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    match kind {
        WorkbenchAlertKind::Inline(tone) => {
            push_inline_alert(commands, node, &rect, clip, order, tone, opacity);
        }
        WorkbenchAlertKind::Toast => {
            push_toast(commands, node, &rect, clip, order, opacity);
        }
    }
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkbenchAlertKind {
    Inline(AlertTone),
    Toast,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AlertTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy)]
struct AlertStyle {
    surface: [u8; 4],
    border: [u8; 4],
    mark: [u8; 4],
    text: [u8; 4],
}

fn workbench_alert_kind(node: &TemplatePaneNodeData) -> Option<WorkbenchAlertKind> {
    match node.control_id.as_str() {
        "WorkbenchInfoAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Info)),
        "WorkbenchSuccessAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Success)),
        "WorkbenchWarningAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Warning)),
        "WorkbenchErrorAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Error)),
        "WorkbenchToastRoot" if is_standalone_toast(node) => Some(WorkbenchAlertKind::Toast),
        "WorkbenchToastRoot" => Some(WorkbenchAlertKind::Inline(
            alert_tone(node).unwrap_or(AlertTone::Info),
        )),
        _ if node.control_id.as_str().starts_with("Workbench")
            && (matches!(node.role.as_str(), "Alert")
                || matches!(node.component_role.as_str(), "alert" | "mui-alert")
                || node.control_id.as_str().ends_with("Alert")) =>
        {
            alert_tone(node).map(WorkbenchAlertKind::Inline)
        }
        _ => None,
    }
}

fn is_standalone_toast(node: &TemplatePaneNodeData) -> bool {
    let label = template_node_label(node, None).to_ascii_lowercase();
    label.contains("operation completed") || label.contains("completed successfully")
}

fn alert_tone(node: &TemplatePaneNodeData) -> Option<AlertTone> {
    let key = format!(
        "{} {} {} {} {} {}",
        node.control_id.as_str(),
        node.icon_name.as_str(),
        node.validation_level.as_str(),
        node.text_tone.as_str(),
        node.component_variant.as_str(),
        template_node_label(node, None)
    )
    .to_ascii_lowercase();
    if key.contains("warning") {
        Some(AlertTone::Warning)
    } else if key.contains("error") || key.contains("danger") || key.contains("failed") {
        Some(AlertTone::Error)
    } else if key.contains("success") || key.contains("check") {
        Some(AlertTone::Success)
    } else if key.contains("info") {
        Some(AlertTone::Info)
    } else {
        None
    }
}

fn push_inline_alert(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tone: AlertTone,
    opacity: f32,
) {
    let style = alert_style(node, tone);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        ALERT_BORDER_WIDTH,
        ALERT_RADIUS,
        opacity,
    ));

    let icon = FrameRect {
        x: rect.x + ALERT_ICON_LEFT,
        y: rect.y + (rect.height - ALERT_ICON_SIZE).max(0.0) * 0.5,
        width: ALERT_ICON_SIZE,
        height: ALERT_ICON_SIZE,
    };
    push_alert_mark(commands, &icon, clip, order + 1, tone, style.mark, opacity);

    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let text_left = icon.x + icon.width + ALERT_TEXT_GAP;
    let text_right = rect.x + rect.width - ALERT_TEXT_RIGHT_INSET;
    if text_right <= text_left {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: text_left,
            y: rect.y + (rect.height - ALERT_LINE_HEIGHT).max(0.0) * 0.5,
            width: text_right - text_left,
            height: ALERT_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order + 2,
        label,
        style.text,
        ALERT_FONT_SIZE,
        ALERT_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_toast(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = select_workbench_toast_style(node);
    let icon_size = toast_status_mark_size(node);

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        1.0,
        TOAST_RADIUS,
        opacity,
    ));

    let icon = FrameRect {
        x: rect.x + TOAST_ICON_LEFT,
        y: rect.y + (rect.height - icon_size).max(0.0) * 0.5,
        width: icon_size,
        height: icon_size,
    };
    push_alert_mark(
        commands,
        &icon,
        clip,
        order + 1,
        AlertTone::Success,
        style.mark,
        opacity,
    );

    let has_action = rect.width >= 210.0;
    let close = toast_close_rect(rect);
    let action_left = close.x - TOAST_ACTION_WIDTH;
    let text_right = if has_action {
        action_left - 4.0
    } else {
        rect.x + rect.width - TOAST_TRAILING_INSET
    };
    let text_left = icon.x + icon.width + TOAST_TEXT_GAP;
    let label = template_node_label(node, None);
    if !label.trim().is_empty() && text_right > text_left {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: text_left,
                y: rect.y + (rect.height - TOAST_LINE_HEIGHT).max(0.0) * 0.5,
                width: text_right - text_left,
                height: TOAST_LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 2,
            label,
            style.text,
            TOAST_FONT_SIZE,
            TOAST_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    if has_action {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: action_left,
                y: rect.y + (rect.height - TOAST_LINE_HEIGHT).max(0.0) * 0.5,
                width: TOAST_ACTION_WIDTH,
                height: TOAST_LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 2,
            TOAST_ACTION_TEXT.to_string(),
            style.action,
            TOAST_FONT_SIZE,
            TOAST_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
        push_close_mark(commands, &close, clip, order + 3, style.close, opacity);
    }
}

fn toast_status_mark_size(node: &TemplatePaneNodeData) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        TOAST_ICON_SIZE
    }
}

fn push_alert_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tone: AlertTone,
    color: [u8; 4],
    opacity: f32,
) {
    match tone {
        AlertTone::Info => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(color),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_segments(
                commands,
                rect,
                clip,
                order + 1,
                ALERT_GLYPH_DARK,
                opacity,
                &[(8.0, 4.0, 2.0, 2.0), (8.0, 8.0, 2.0, 6.0)],
            );
        }
        AlertTone::Success => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(color),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_segments(
                commands,
                rect,
                clip,
                order + 1,
                ALERT_GLYPH_DARK,
                opacity,
                &[
                    (4.0, 9.0, 3.0, 2.0),
                    (6.0, 11.0, 3.0, 2.0),
                    (9.0, 6.0, 3.0, 7.0),
                ],
            );
        }
        AlertTone::Warning => {
            let center_x = rect.x + rect.width * 0.5;
            for (row, width) in [3.0, 5.0, 7.0, 9.0, 11.0, 13.0].into_iter().enumerate() {
                commands.push(HostPaintCommand::quad(
                    FrameRect {
                        x: center_x - width * 0.5,
                        y: rect.y + 3.0 + row as f32 * 1.85,
                        width,
                        height: 2.0,
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
                rect,
                clip,
                order + 1,
                ALERT_GLYPH_DARK,
                opacity,
                &[(8.0, 8.0, 2.0, 4.0), (8.0, 14.0, 2.0, 2.0)],
            );
        }
        AlertTone::Error => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(color),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_close_mark(commands, rect, clip, order + 1, ALERT_GLYPH_DARK, opacity);
        }
    }
}

fn push_close_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 4.0, 2.0, 2.0),
            (6.0, 6.0, 2.0, 2.0),
            (8.0, 8.0, 2.0, 2.0),
            (10.0, 10.0, 2.0, 2.0),
            (10.0, 4.0, 2.0, 2.0),
            (8.0, 6.0, 2.0, 2.0),
            (6.0, 8.0, 2.0, 2.0),
            (4.0, 10.0, 2.0, 2.0),
        ],
    );
}

fn alert_style(node: &TemplatePaneNodeData, tone: AlertTone) -> AlertStyle {
    let disabled = node.disabled;
    let mark = if disabled {
        PALETTE.text_disabled
    } else {
        alert_mark_color(tone)
    };
    let text = if disabled {
        PALETTE.text_disabled
    } else {
        mark
    };
    match tone {
        AlertTone::Info => AlertStyle {
            surface: if disabled {
                PALETTE.surface_disabled
            } else {
                INFO_SURFACE
            },
            border: if disabled {
                PALETTE.border_disabled
            } else {
                INFO_BORDER
            },
            mark,
            text,
        },
        AlertTone::Success => AlertStyle {
            surface: if disabled {
                PALETTE.surface_disabled
            } else {
                SUCCESS_SURFACE
            },
            border: if disabled {
                PALETTE.border_disabled
            } else {
                SUCCESS_BORDER
            },
            mark,
            text,
        },
        AlertTone::Warning => AlertStyle {
            surface: if disabled {
                PALETTE.surface_disabled
            } else {
                WARNING_SURFACE
            },
            border: if disabled {
                PALETTE.border_disabled
            } else {
                WARNING_BORDER
            },
            mark,
            text,
        },
        AlertTone::Error => AlertStyle {
            surface: if disabled {
                PALETTE.surface_disabled
            } else {
                ERROR_SURFACE
            },
            border: if disabled {
                PALETTE.border_disabled
            } else {
                ERROR_BORDER
            },
            mark,
            text,
        },
    }
}

fn alert_mark_color(tone: AlertTone) -> [u8; 4] {
    match tone {
        AlertTone::Info => PALETTE.info,
        AlertTone::Success => PALETTE.success,
        AlertTone::Warning => PALETTE.warning,
        AlertTone::Error => PALETTE.error,
    }
}

fn toast_close_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - TOAST_TRAILING_INSET - TOAST_CLOSE_SIZE,
        y: rect.y + (rect.height - TOAST_CLOSE_SIZE).max(0.0) * 0.5,
        width: TOAST_CLOSE_SIZE,
        height: TOAST_CLOSE_SIZE,
    }
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[(f32, f32, f32, f32)],
) {
    for (x, y, width, height) in segments {
        commands.push(HostPaintCommand::quad(
            scaled_rect(origin, *x, *y, *width, *height),
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

fn scaled_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    let scale_x = origin.width / ALERT_ICON_SIZE;
    let scale_y = origin.height / ALERT_ICON_SIZE;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
    }
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::style_selector::{
        WORKBENCH_TOAST_ACTION, WORKBENCH_TOAST_BORDER, WORKBENCH_TOAST_SURFACE,
    };
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use zircon_runtime_interface::ui::style::UiPainterResolvedState;

    #[test]
    fn workbench_alert_kind_matches_drawer_ids_and_toast_root() {
        assert_eq!(
            workbench_alert_kind(&alert_node("WorkbenchInfoAlert", "Info Alert", "info")),
            Some(WorkbenchAlertKind::Inline(AlertTone::Info))
        );
        assert_eq!(
            workbench_alert_kind(&alert_node("WorkbenchErrorAlert", "Error Alert", "error")),
            Some(WorkbenchAlertKind::Inline(AlertTone::Error))
        );
        assert_eq!(
            workbench_alert_kind(&alert_node(
                "WorkbenchToastRoot",
                "Operation completed successfully",
                "info"
            )),
            Some(WorkbenchAlertKind::Toast)
        );
        assert_eq!(
            workbench_alert_kind(&alert_node("PlainAlert", "Info Alert", "info")),
            None
        );
    }

    #[test]
    fn workbench_info_alert_paints_tinted_surface_icon_and_label() {
        let bytes = paint_template_nodes_for_test(
            192,
            48,
            model_rc(vec![positioned_alert_node(
                "WorkbenchInfoAlert",
                "Info Alert",
                "info",
                8.0,
                8.0,
                160.0,
                32.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 192, 80, 24), INFO_SURFACE);
        assert_eq!(pixel_at(&bytes, 192, 25, 24), PALETTE.info);
        assert!(changed_pixel_count(&bytes, 192, 38, 16, 62, 18) > 0);
        assert_eq!(pixel_at(&bytes, 192, 176, 24), [0, 0, 0, 255]);
    }

    #[test]
    fn workbench_warning_alert_uses_warning_tone() {
        let bytes = paint_template_nodes_for_test(
            192,
            48,
            model_rc(vec![positioned_alert_node(
                "WorkbenchWarningAlert",
                "Warning Alert",
                "warning",
                8.0,
                8.0,
                160.0,
                32.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 192, 150, 24), WARNING_SURFACE);
        assert_eq!(pixel_at(&bytes, 192, 27, 18), PALETTE.warning);
        assert!(changed_pixel_count(&bytes, 192, 38, 16, 84, 18) > 0);
    }

    #[test]
    fn workbench_toast_paints_status_mark_action_and_close() {
        let bytes = paint_template_nodes_for_test(
            320,
            48,
            model_rc(vec![positioned_alert_node(
                "WorkbenchToastRoot",
                "Operation completed successfully",
                "success",
                8.0,
                8.0,
                280.0,
                32.0,
            )]),
        );

        let surface_pixel = blend_over(WORKBENCH_TOAST_SURFACE, [0, 0, 0, 255]);
        assert_eq!(WORKBENCH_TOAST_SURFACE, [21, 48, 53, 247]);
        assert_eq!(WORKBENCH_TOAST_BORDER, [53, 199, 208, 20]);
        assert_eq!(pixel_at(&bytes, 320, 160, 12), surface_pixel);
        assert_eq!(
            pixel_at(&bytes, 320, 160, 8),
            blend_over(WORKBENCH_TOAST_BORDER, surface_pixel)
        );
        assert_ne!(pixel_at(&bytes, 320, 120, 24), [0, 0, 0, 255]);
        assert_eq!(pixel_at(&bytes, 320, 35, 24), WORKBENCH_TOAST_ACTION);
        assert!(changed_pixel_count(&bytes, 320, 233, 16, 34, 18) > 0);
        assert!(changed_pixel_count(&bytes, 320, 269, 17, 12, 14) > 0);
    }

    #[test]
    fn workbench_toast_uses_declared_status_mark_and_action_style() {
        let mut node = positioned_alert_node(
            "WorkbenchToastRoot",
            "Operation completed successfully",
            "success",
            8.0,
            8.0,
            280.0,
            32.0,
        );
        node.value_number = 12.0;
        node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(32, 159, 169);
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(35, 143, 152);

        let style = select_workbench_toast_style(&node);
        assert_eq!(toast_status_mark_size(&node), 12.0);
        assert_eq!(style.mark, [32, 159, 169, 255]);
        assert_eq!(style.action, [35, 143, 152, 255]);

        let bytes = paint_template_nodes_for_test(320, 48, model_rc(vec![node]));
        assert_eq!(pixel_at(&bytes, 320, 22, 20), [32, 159, 169, 255]);
    }

    #[test]
    fn workbench_toast_style_uses_shared_state_priority() {
        let mut node = positioned_alert_node(
            "WorkbenchToastRoot",
            "Operation completed successfully",
            "success",
            8.0,
            8.0,
            280.0,
            32.0,
        );
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.disabled = true;

        let disabled = select_workbench_toast_style(&node);
        assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
        assert_eq!(disabled.action, PALETTE.text_disabled);

        node.disabled = false;
        let pressed = select_workbench_toast_style(&node);
        assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
        assert_eq!(pressed.border, PALETTE.focus_ring);

        node.pressed = false;
        let focused = select_workbench_toast_style(&node);
        assert_eq!(focused.state, UiPainterResolvedState::Focused);
    }

    fn alert_node(control_id: &str, text: &str, tone: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Alert".into(),
            component_role: "alert".into(),
            text: text.into(),
            validation_level: tone.into(),
            icon_name: tone.into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 160.0,
                height: 32.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn positioned_alert_node(
        control_id: &str,
        text: &str,
        tone: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            frame: TemplateNodeFrameData {
                x,
                y,
                width,
                height,
            },
            ..alert_node(control_id, text, tone)
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

    fn blend_over(source: [u8; 4], destination: [u8; 4]) -> [u8; 4] {
        let alpha = source[3] as u32;
        let inverse = 255 - alpha;
        [
            ((source[0] as u32 * alpha + destination[0] as u32 * inverse) / 255) as u8,
            ((source[1] as u32 * alpha + destination[1] as u32 * inverse) / 255) as u8,
            ((source[2] as u32 * alpha + destination[2] as u32 * inverse) / 255) as u8,
            255,
        ]
    }
}
