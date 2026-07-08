use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

const NOTIFICATIONS: &str = "notifications";
const OPEN: &str = "open";
const POPUP_OPEN: &str = "popup_open";
const POPUP_OPEN_CAMEL: &str = "popupOpen";
const UNREAD_COUNT: &str = "unread_count";
const FOCUSED_INDEX: &str = "focused_index";
const SELECTED_NOTIFICATION_ID: &str = "selected_notification_id";
const VISIBLE_LIMIT: &str = "visible_limit";
const TITLE: &str = "title";
const EMPTY_TEXT: &str = "empty_text";

const PANEL_SURFACE: &str = "#11181d";
const PANEL_BORDER: &str = "#2d3a42";
const HEADER_TEXT: &str = "#e7eef0";
const MUTED_TEXT: &str = "#7f8f95";
const ROW_SURFACE: &str = "#151e23";
const ROW_UNREAD_SURFACE: &str = "#153035";
const ROW_DISABLED_SURFACE: &str = "#252c31";
const ROW_BORDER: &str = "#283842";
const ACCENT: &str = "#35c7d0";
const ERROR: &str = "#ef7066";
const SUCCESS: &str = "#42b883";
const WARNING: &str = "#e0a33a";

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

pub(super) fn notification_center_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_notification_center)
}

pub(super) fn notification_center_suppresses_owner_image(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_notification_center)
}

pub(super) fn notification_center_suppresses_owner_surface(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_notification_center)
}

pub(super) fn notification_center_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    state_flags: &UiStateFlags,
    component_state: Option<&UiComponentState>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if !is_notification_center(metadata) || !notification_center_open(metadata, component_state) {
        return Vec::new();
    }
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = NotificationCenterRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        PANEL_SURFACE,
        Some(PANEL_BORDER),
        1.0,
        PANEL_RADIUS,
        state.panel_state,
        opacity,
    )];

    commands.push(text_command(
        node_id,
        UiFrame::new(
            frame.x + PANEL_PADDING_X,
            frame.y + HEADER_TOP,
            (frame.width - PANEL_PADDING_X * 2.0).max(1.0),
            HEADER_HEIGHT,
        ),
        clip_frame,
        z_index.saturating_add(2),
        header_text(metadata),
        HEADER_TEXT,
        HEADER_FONT_SIZE,
        HEADER_LINE_HEIGHT,
        state.panel_state,
        opacity,
    ));

    let rows = notification_rows(metadata);
    if rows.is_empty() {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + PANEL_PADDING_X,
                frame.y + EMPTY_TEXT_TOP,
                (frame.width - PANEL_PADDING_X * 2.0).max(1.0),
                MESSAGE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(3),
            string_attribute(metadata, EMPTY_TEXT)
                .unwrap_or("No notifications")
                .to_string(),
            MUTED_TEXT,
            MESSAGE_FONT_SIZE,
            MESSAGE_LINE_HEIGHT,
            UiPainterResolvedState::Normal,
            opacity,
        ));
        return commands;
    }

    for (index, row) in rows.iter().enumerate() {
        let row_frame = UiFrame::new(
            frame.x + ROW_INSET_X,
            frame.y + ROW_TOP + index as f32 * (ROW_HEIGHT + ROW_GAP),
            (frame.width - ROW_INSET_X * 2.0).max(1.0),
            ROW_HEIGHT,
        );
        let row_state = row.paint_state();
        let row_z = z_index.saturating_add(3 + index as i32 * 4);

        commands.push(quad_command(
            node_id,
            row_frame,
            clip_frame,
            row_z,
            row.background(),
            Some(row.border()),
            1.0,
            4.0,
            row_state,
            opacity,
        ));
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                row_frame.x + MARK_LEFT,
                row_frame.y + MARK_TOP,
                MARK_WIDTH,
                MARK_HEIGHT,
            ),
            clip_frame,
            row_z.saturating_add(1),
            row.severity_color(),
            None,
            0.0,
            1.0,
            row_state,
            opacity,
        ));

        let text_frame_width = (row_frame.width - TEXT_LEFT - TEXT_RIGHT_INSET).max(1.0);
        commands.push(text_command(
            node_id,
            UiFrame::new(
                row_frame.x + TEXT_LEFT,
                row_frame.y + TITLE_TOP,
                text_frame_width,
                TITLE_HEIGHT,
            ),
            clip_frame,
            row_z.saturating_add(2),
            row.title.clone(),
            row.title_color(),
            TITLE_FONT_SIZE,
            TITLE_LINE_HEIGHT,
            row_state,
            opacity,
        ));

        if !row.message.is_empty() {
            commands.push(text_command(
                node_id,
                UiFrame::new(
                    row_frame.x + TEXT_LEFT,
                    row_frame.y + MESSAGE_TOP,
                    text_frame_width,
                    MESSAGE_HEIGHT,
                ),
                clip_frame,
                row_z.saturating_add(3),
                row.message.clone(),
                MUTED_TEXT,
                MESSAGE_FONT_SIZE,
                MESSAGE_LINE_HEIGHT,
                row_state,
                opacity,
            ));
        }
    }

    commands
}

#[derive(Clone, Copy)]
struct NotificationCenterRenderState {
    panel_state: UiPainterResolvedState,
}

impl NotificationCenterRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        Self {
            panel_state: painter_state.resolved_state_for_family(UiPainterFamily::Toast),
        }
    }
}

#[derive(Clone, Debug)]
struct NotificationRow {
    id: String,
    title: String,
    message: String,
    severity: NotificationSeverity,
    unread: bool,
    disabled: bool,
    selected: bool,
    focused: bool,
}

impl NotificationRow {
    fn new(id: String) -> Self {
        Self {
            title: id.clone(),
            id,
            message: String::new(),
            severity: NotificationSeverity::Info,
            unread: false,
            disabled: false,
            selected: false,
            focused: false,
        }
    }

    fn matches_id(&self, id: &str) -> bool {
        !id.is_empty() && (self.id == id || self.title == id)
    }

    fn paint_state(&self) -> UiPainterResolvedState {
        if self.disabled {
            UiPainterResolvedState::Disabled
        } else if self.selected {
            UiPainterResolvedState::Selected
        } else if self.focused {
            UiPainterResolvedState::Focused
        } else {
            UiPainterResolvedState::Normal
        }
    }

    fn background(&self) -> &'static str {
        if self.disabled {
            ROW_DISABLED_SURFACE
        } else if self.unread {
            ROW_UNREAD_SURFACE
        } else {
            ROW_SURFACE
        }
    }

    fn border(&self) -> &'static str {
        if self.selected || self.focused {
            ACCENT
        } else {
            ROW_BORDER
        }
    }

    fn severity_color(&self) -> &'static str {
        self.severity.color()
    }

    fn title_color(&self) -> &'static str {
        if self.disabled {
            MUTED_TEXT
        } else {
            HEADER_TEXT
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum NotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationSeverity {
    fn from_str(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "success" | "ok" | "done" => Self::Success,
            "warning" | "warn" => Self::Warning,
            "error" | "danger" | "failed" | "failure" => Self::Error,
            _ => Self::Info,
        }
    }

    fn color(self) -> &'static str {
        match self {
            Self::Info => ACCENT,
            Self::Success => SUCCESS,
            Self::Warning => WARNING,
            Self::Error => ERROR,
        }
    }
}

fn is_notification_center(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "NotificationCenter"
}

fn notification_center_open(
    metadata: &UiTemplateNodeMetadata,
    component_state: Option<&UiComponentState>,
) -> bool {
    bool_attribute(metadata, POPUP_OPEN).unwrap_or(false)
        || bool_attribute(metadata, POPUP_OPEN_CAMEL).unwrap_or(false)
        || bool_attribute(metadata, OPEN).unwrap_or(false)
        || component_state.is_some_and(|state| state.flags.popup_open)
}

fn header_text(metadata: &UiTemplateNodeMetadata) -> String {
    let title = string_attribute(metadata, TITLE)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Notifications");
    let unread_count = usize_attribute(metadata, UNREAD_COUNT).unwrap_or(0);
    if unread_count > 0 {
        format!("{title} ({unread_count})")
    } else {
        title.to_string()
    }
}

fn notification_rows(metadata: &UiTemplateNodeMetadata) -> Vec<NotificationRow> {
    let selected_id = string_attribute(metadata, SELECTED_NOTIFICATION_ID).unwrap_or_default();
    let focused_index = usize_attribute(metadata, FOCUSED_INDEX);
    let visible_limit = usize_attribute(metadata, VISIBLE_LIMIT).unwrap_or(usize::MAX);
    let mut rows = metadata
        .attributes
        .get(NOTIFICATIONS)
        .map(notification_entry_list)
        .unwrap_or_default();

    for (index, row) in rows.iter_mut().enumerate() {
        row.selected = row.matches_id(selected_id);
        row.focused = focused_index == Some(index);
    }

    rows.into_iter().take(visible_limit).collect()
}

fn notification_entry_list(value: &Value) -> Vec<NotificationRow> {
    match value {
        Value::Array(values) => values.iter().flat_map(notification_entry_list).collect(),
        Value::String(value) => notification_entry_from_string(value).into_iter().collect(),
        Value::Table(values) => notification_entry_from_table(values).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn notification_entry_from_string(value: &str) -> Option<NotificationRow> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() {
        return None;
    }

    let mut row = NotificationRow::new(id);
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "title" | "label" | "text" | "name" => row.title = value.to_string(),
            "message" | "body" | "description" | "detail" => row.message = value.to_string(),
            "severity" | "level" | "kind" => row.severity = NotificationSeverity::from_str(value),
            "unread" | "new" => row.unread = string_bool(value).unwrap_or(false),
            "disabled" => row.disabled = string_bool(value).unwrap_or(false),
            "enabled" => row.disabled = string_bool(value) == Some(false),
            _ => {}
        }
    }
    Some(row)
}

fn notification_entry_from_table(
    values: &toml::map::Map<String, Value>,
) -> Option<NotificationRow> {
    let id = first_string_value(
        values,
        &["id", "notification_id", "notificationId", "value", "key"],
    )?;
    if id.is_empty() {
        return None;
    }

    Some(NotificationRow {
        title: first_string_value(values, &["title", "label", "text", "name"])
            .unwrap_or_else(|| id.clone()),
        message: first_string_value(values, &["message", "body", "description", "detail"])
            .unwrap_or_default(),
        severity: first_string_value(values, &["severity", "level", "kind"])
            .map(|value| NotificationSeverity::from_str(&value))
            .unwrap_or(NotificationSeverity::Info),
        unread: values
            .get("unread")
            .or_else(|| values.get("new"))
            .and_then(bool_value)
            .unwrap_or(false),
        disabled: values.get("disabled").and_then(bool_value).unwrap_or(false)
            || values.get("enabled").and_then(bool_value) == Some(false),
        selected: false,
        focused: false,
        id,
    })
}

fn first_string_value(values: &toml::map::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    match metadata.attributes.get(key)? {
        Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    }
}

fn string_value(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn bool_value(value: &Value) -> Option<bool> {
    match value {
        Value::Boolean(value) => Some(*value),
        Value::String(value) => string_bool(value),
        _ => None,
    }
}

fn string_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: &str,
    border: Option<&str>,
    border_width: f32,
    corner_radius: f32,
    painter_state: UiPainterResolvedState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(background.to_string()),
            border_color: border.map(str::to_string),
            border_width,
            corner_radius,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Toast, painter_state),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: &str,
    font_size: f32,
    line_height: f32,
    painter_state: UiPainterResolvedState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(foreground.to_string()),
            font_size,
            line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Toast, painter_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}
