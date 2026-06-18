use super::super::data::{FrameRect, TemplatePaneNodeData, TemplatePaneOptionData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const PANEL_SURFACE: [u8; 4] = [17, 24, 29, 255];
const PANEL_BORDER: [u8; 4] = [45, 58, 66, 255];
const HEADER_TEXT: [u8; 4] = [231, 238, 240, 255];
const MUTED_TEXT: [u8; 4] = [127, 143, 149, 255];
const ROW_SURFACE: [u8; 4] = [21, 30, 35, 255];
const ROW_UNREAD_SURFACE: [u8; 4] = [21, 48, 53, 255];
const ROW_FOCUSED_SURFACE: [u8; 4] = [24, 58, 63, 255];
const ROW_DISABLED_SURFACE: [u8; 4] = [37, 44, 49, 255];
const ROW_BORDER: [u8; 4] = [40, 56, 66, 255];
const ACCENT: [u8; 4] = [53, 199, 208, 255];
const ERROR: [u8; 4] = [239, 112, 102, 255];
const SUCCESS: [u8; 4] = [66, 184, 131, 255];
const WARNING: [u8; 4] = [224, 163, 58, 255];

const PANEL_RADIUS: f32 = 6.0;
const PANEL_PADDING_X: f32 = 12.0;
const HEADER_TOP: f32 = 10.0;
const HEADER_HEIGHT: f32 = 16.0;
const ROW_INSET_X: f32 = 8.0;
const ROW_TOP: f32 = 36.0;
const ROW_HEIGHT: f32 = 48.0;
const ROW_GAP: f32 = 6.0;
const MARK_LEFT: f32 = 10.0;
const MARK_TOP: f32 = 8.0;
const MARK_WIDTH: f32 = 3.0;
const MARK_HEIGHT: f32 = 32.0;
const TEXT_LEFT: f32 = 22.0;
const TEXT_RIGHT_INSET: f32 = 12.0;
const TITLE_TOP: f32 = 7.0;
const TITLE_HEIGHT: f32 = 14.0;
const MESSAGE_TOP: f32 = 25.0;
const MESSAGE_HEIGHT: f32 = 13.0;
const HEADER_FONT_SIZE: f32 = 13.0;
const HEADER_LINE_HEIGHT: f32 = 16.0;
const TITLE_FONT_SIZE: f32 = 12.0;
const TITLE_LINE_HEIGHT: f32 = 14.0;
const MESSAGE_FONT_SIZE: f32 = 11.0;
const MESSAGE_LINE_HEIGHT: f32 = 13.0;
const EMPTY_TEXT_TOP: f32 = 48.0;

pub(super) fn push_notification_center_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_notification_center(node) {
        return false;
    }
    if !node.popup_open {
        return true;
    }

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 1.0 || rect.height <= 1.0 {
        return true;
    }

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(PANEL_SURFACE),
        Some(PANEL_BORDER),
        1.0,
        PANEL_RADIUS,
        opacity,
    ));

    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + PANEL_PADDING_X,
            y: rect.y + HEADER_TOP,
            width: (rect.width - PANEL_PADDING_X * 2.0).max(1.0),
            height: HEADER_HEIGHT,
        },
        Some(clip.clone()),
        order + 1,
        header_text(node),
        HEADER_TEXT,
        HEADER_FONT_SIZE,
        HEADER_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));

    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: rect.x + PANEL_PADDING_X,
                y: rect.y + EMPTY_TEXT_TOP,
                width: (rect.width - PANEL_PADDING_X * 2.0).max(1.0),
                height: MESSAGE_HEIGHT,
            },
            Some(clip.clone()),
            order + 2,
            empty_text(node),
            MUTED_TEXT,
            MESSAGE_FONT_SIZE,
            MESSAGE_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
        return true;
    }

    for row in 0..row_count {
        let Some(option) = node.structured_options.row_data(row) else {
            continue;
        };
        let row_rect = FrameRect {
            x: rect.x + ROW_INSET_X,
            y: rect.y + ROW_TOP + row as f32 * (ROW_HEIGHT + ROW_GAP),
            width: (rect.width - ROW_INSET_X * 2.0).max(1.0),
            height: ROW_HEIGHT,
        };
        push_notification_row(
            commands,
            &option,
            &row_rect,
            clip,
            order + 3 + row as i32 * 4,
            opacity,
        );
    }

    true
}

fn is_notification_center(node: &TemplatePaneNodeData) -> bool {
    node.role.as_str() == "NotificationCenter"
        || node.component_role.as_str() == "notification-center"
}

fn header_text(node: &TemplatePaneNodeData) -> String {
    let title = non_empty(node.text.as_str()).unwrap_or("Notifications");
    let unread_count = (0..node.structured_options.row_count())
        .filter_map(|row| node.structured_options.row_data(row))
        .filter(|option| option.unread)
        .count();
    if unread_count > 0 {
        format!("{title} ({unread_count})")
    } else {
        title.to_string()
    }
}

fn empty_text(node: &TemplatePaneNodeData) -> String {
    non_empty(node.value_text.as_str())
        .unwrap_or("No notifications")
        .to_string()
}

fn push_notification_row(
    commands: &mut Vec<HostPaintCommand>,
    option: &TemplatePaneOptionData,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }

    commands.push(HostPaintCommand::quad(
        row_rect.clone(),
        Some(clip.clone()),
        order,
        Some(row_background(option)),
        Some(row_border(option)),
        1.0,
        4.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: row_rect.x + MARK_LEFT,
            y: row_rect.y + MARK_TOP,
            width: MARK_WIDTH,
            height: MARK_HEIGHT,
        },
        Some(clip.clone()),
        order + 1,
        Some(severity_color(option.tone.as_str())),
        None,
        0.0,
        1.0,
        opacity,
    ));

    let text_width = (row_rect.width - TEXT_LEFT - TEXT_RIGHT_INSET).max(1.0);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: row_rect.x + TEXT_LEFT,
            y: row_rect.y + TITLE_TOP,
            width: text_width,
            height: TITLE_HEIGHT,
        },
        Some(clip.clone()),
        order + 2,
        option.label.to_string(),
        title_color(option),
        TITLE_FONT_SIZE,
        TITLE_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));

    let message = option.description.to_string();
    if message.is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: row_rect.x + TEXT_LEFT,
            y: row_rect.y + MESSAGE_TOP,
            width: text_width,
            height: MESSAGE_HEIGHT,
        },
        Some(clip.clone()),
        order + 3,
        message,
        MUTED_TEXT,
        MESSAGE_FONT_SIZE,
        MESSAGE_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn row_background(option: &TemplatePaneOptionData) -> [u8; 4] {
    if option.disabled {
        ROW_DISABLED_SURFACE
    } else if option.focused {
        ROW_FOCUSED_SURFACE
    } else if option.unread {
        ROW_UNREAD_SURFACE
    } else {
        ROW_SURFACE
    }
}

fn row_border(option: &TemplatePaneOptionData) -> [u8; 4] {
    if option.selected {
        ACCENT
    } else {
        ROW_BORDER
    }
}

fn title_color(option: &TemplatePaneOptionData) -> [u8; 4] {
    if option.disabled {
        MUTED_TEXT
    } else {
        HEADER_TEXT
    }
}

fn severity_color(tone: &str) -> [u8; 4] {
    match tone {
        "success" => SUCCESS,
        "warning" => WARNING,
        "error" => ERROR,
        _ => ACCENT,
    }
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
