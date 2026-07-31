use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::EditorDesignTokens,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
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
const PANEL_SURFACE_COLOR: &str = "panel_surface_color";
const PANEL_BORDER_COLOR: &str = "panel_border_color";
const ROW_SURFACE_COLOR: &str = "row_surface_color";
const ROW_UNREAD_SURFACE_COLOR: &str = "row_unread_surface_color";
const ROW_DISABLED_SURFACE_COLOR: &str = "row_disabled_surface_color";
const ROW_BORDER_COLOR: &str = "row_border_color";
const HEADER_TEXT_COLOR: &str = "header_text_color";
const MUTED_TEXT_COLOR: &str = "muted_text_color";
const ACCENT_COLOR: &str = "accent_color";
const SUCCESS_COLOR: &str = "success_color";
const WARNING_COLOR: &str = "warning_color";
const ERROR_COLOR: &str = "error_color";
const PANEL_BORDER_WIDTH: &str = "panel_border_width";
const PANEL_RADIUS: &str = "panel_radius";
const ROW_RADIUS: &str = "row_radius";
const MARK_RADIUS: &str = "mark_radius";
const HEADER_FONT_SIZE: &str = "header_font_size";
const TITLE_FONT_SIZE: &str = "title_font_size";
const MESSAGE_FONT_SIZE: &str = "message_font_size";
const TYPOGRAPHY_LINE_HEIGHT_RATIO: &str = "typography_line_height_ratio";

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
const MESSAGE_TOP: f32 = 25.0;
const EMPTY_TEXT_TOP: f32 = 48.0;

#[derive(Clone, Copy)]
struct NotificationCenterVisual {
    panel_surface: UiRgbaColor,
    panel_border: UiRgbaColor,
    row_surface: UiRgbaColor,
    row_unread_surface: UiRgbaColor,
    row_disabled_surface: UiRgbaColor,
    row_border: UiRgbaColor,
    header_text: UiRgbaColor,
    muted_text: UiRgbaColor,
    accent: UiRgbaColor,
    error: UiRgbaColor,
    success: UiRgbaColor,
    warning: UiRgbaColor,
    border_width: f32,
    panel_radius: f32,
    row_radius: f32,
    mark_radius: f32,
    header_font_size: f32,
    header_line_height: f32,
    title_font_size: f32,
    title_line_height: f32,
    message_font_size: f32,
    message_line_height: f32,
}

impl NotificationCenterVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_notification_center_visual();
        visual.panel_surface =
            rgba_attribute(metadata, PANEL_SURFACE_COLOR).unwrap_or(visual.panel_surface);
        visual.panel_border =
            rgba_attribute(metadata, PANEL_BORDER_COLOR).unwrap_or(visual.panel_border);
        visual.row_surface =
            rgba_attribute(metadata, ROW_SURFACE_COLOR).unwrap_or(visual.row_surface);
        visual.row_unread_surface =
            rgba_attribute(metadata, ROW_UNREAD_SURFACE_COLOR).unwrap_or(visual.row_unread_surface);
        visual.row_disabled_surface = rgba_attribute(metadata, ROW_DISABLED_SURFACE_COLOR)
            .unwrap_or(visual.row_disabled_surface);
        visual.row_border = rgba_attribute(metadata, ROW_BORDER_COLOR).unwrap_or(visual.row_border);
        visual.header_text =
            rgba_attribute(metadata, HEADER_TEXT_COLOR).unwrap_or(visual.header_text);
        visual.muted_text = rgba_attribute(metadata, MUTED_TEXT_COLOR).unwrap_or(visual.muted_text);
        visual.accent = rgba_attribute(metadata, ACCENT_COLOR).unwrap_or(visual.accent);
        visual.success = rgba_attribute(metadata, SUCCESS_COLOR).unwrap_or(visual.success);
        visual.warning = rgba_attribute(metadata, WARNING_COLOR).unwrap_or(visual.warning);
        visual.error = rgba_attribute(metadata, ERROR_COLOR).unwrap_or(visual.error);
        visual.border_width =
            positive_float_attribute(metadata, PANEL_BORDER_WIDTH).unwrap_or(visual.border_width);
        visual.panel_radius =
            nonnegative_float_attribute(metadata, PANEL_RADIUS).unwrap_or(visual.panel_radius);
        visual.row_radius =
            nonnegative_float_attribute(metadata, ROW_RADIUS).unwrap_or(visual.row_radius);
        visual.mark_radius =
            nonnegative_float_attribute(metadata, MARK_RADIUS).unwrap_or(visual.mark_radius);
        visual.header_font_size =
            positive_float_attribute(metadata, HEADER_FONT_SIZE).unwrap_or(visual.header_font_size);
        visual.title_font_size =
            positive_float_attribute(metadata, TITLE_FONT_SIZE).unwrap_or(visual.title_font_size);
        visual.message_font_size = positive_float_attribute(metadata, MESSAGE_FONT_SIZE)
            .unwrap_or(visual.message_font_size);

        let line_height_ratio = positive_float_attribute(metadata, TYPOGRAPHY_LINE_HEIGHT_RATIO)
            .unwrap_or(visual.header_line_height / visual.header_font_size);
        visual.header_line_height = visual.header_font_size * line_height_ratio;
        visual.title_line_height = visual.title_font_size * line_height_ratio;
        visual.message_line_height = visual.message_font_size * line_height_ratio;
        visual
    }
}

fn default_notification_center_visual() -> &'static NotificationCenterVisual {
    static VISUAL: OnceLock<NotificationCenterVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let palette = &tokens.palette;
        let typography = &tokens.typography;
        let controls = &tokens.controls;

        NotificationCenterVisual {
            panel_surface: palette.popup,
            panel_border: palette.border,
            row_surface: palette.surface[1],
            row_unread_surface: palette.surface_selected,
            row_disabled_surface: palette.surface_disabled,
            row_border: palette.separator_soft,
            header_text: palette.text_primary,
            muted_text: palette.text_secondary,
            accent: palette.accent,
            error: palette.error,
            success: palette.success,
            warning: palette.warning,
            border_width: controls.border_width,
            panel_radius: controls.panel_radius,
            row_radius: controls.small_radius,
            mark_radius: controls.border_width,
            header_font_size: typography.body_size,
            header_line_height: typography.body_size * typography.line_height,
            title_font_size: typography.overlay_size,
            title_line_height: typography.overlay_size * typography.line_height,
            message_font_size: typography.caption_size,
            message_line_height: typography.caption_size * typography.line_height,
        }
    })
}

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
    let visual = NotificationCenterVisual::resolve(metadata);
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        visual.panel_surface,
        Some(visual.panel_border),
        visual.border_width,
        visual.panel_radius,
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
        visual.header_text,
        visual.header_font_size,
        visual.header_line_height,
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
                visual.message_line_height,
            ),
            clip_frame,
            z_index.saturating_add(3),
            string_attribute(metadata, EMPTY_TEXT)
                .unwrap_or("No notifications")
                .to_string(),
            visual.muted_text,
            visual.message_font_size,
            visual.message_line_height,
            UiPainterResolvedState::Normal,
            opacity,
        ));
        return commands;
    }

    for (index, row) in rows.into_iter().enumerate() {
        let row_frame = UiFrame::new(
            frame.x + ROW_INSET_X,
            frame.y + ROW_TOP + index as f32 * (ROW_HEIGHT + ROW_GAP),
            (frame.width - ROW_INSET_X * 2.0).max(1.0),
            ROW_HEIGHT,
        );
        let row_state = row.paint_state();
        let row_z = z_index.saturating_add(3 + index as i32 * 4);
        let background = row.background(&visual);
        let border = row.border(&visual);
        let severity_color = row.severity_color(&visual);
        let title_color = row.title_color(&visual);
        let title = row.title;
        let message = row.message;

        commands.push(quad_command(
            node_id,
            row_frame,
            clip_frame,
            row_z,
            background,
            Some(border),
            visual.border_width,
            visual.row_radius,
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
            severity_color,
            None,
            0.0,
            visual.mark_radius,
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
                visual.title_line_height,
            ),
            clip_frame,
            row_z.saturating_add(2),
            title,
            title_color,
            visual.title_font_size,
            visual.title_line_height,
            row_state,
            opacity,
        ));

        if !message.is_empty() {
            commands.push(text_command(
                node_id,
                UiFrame::new(
                    row_frame.x + TEXT_LEFT,
                    row_frame.y + MESSAGE_TOP,
                    text_frame_width,
                    visual.message_line_height,
                ),
                clip_frame,
                row_z.saturating_add(3),
                message,
                visual.muted_text,
                visual.message_font_size,
                visual.message_line_height,
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

    fn background(&self, visual: &NotificationCenterVisual) -> UiRgbaColor {
        if self.disabled {
            visual.row_disabled_surface
        } else if self.unread {
            visual.row_unread_surface
        } else {
            visual.row_surface
        }
    }

    fn border(&self, visual: &NotificationCenterVisual) -> UiRgbaColor {
        if self.selected || self.focused {
            visual.accent
        } else {
            visual.row_border
        }
    }

    fn severity_color(&self, visual: &NotificationCenterVisual) -> UiRgbaColor {
        self.severity.color(visual)
    }

    fn title_color(&self, visual: &NotificationCenterVisual) -> UiRgbaColor {
        if self.disabled {
            visual.muted_text
        } else {
            visual.header_text
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
        let value = value.trim();
        if matches_ascii_alias(value, &["success", "ok", "done"]) {
            Self::Success
        } else if matches_ascii_alias(value, &["warning", "warn"]) {
            Self::Warning
        } else if matches_ascii_alias(value, &["error", "danger", "failed", "failure"]) {
            Self::Error
        } else {
            Self::Info
        }
    }

    fn color(self, visual: &NotificationCenterVisual) -> UiRgbaColor {
        match self {
            Self::Info => visual.accent,
            Self::Success => visual.success,
            Self::Warning => visual.warning,
            Self::Error => visual.error,
        }
    }
}

fn matches_ascii_alias(value: &str, aliases: &[&str]) -> bool {
    aliases
        .iter()
        .any(|alias| value.eq_ignore_ascii_case(alias))
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

fn rgba_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<UiRgbaColor> {
    let encoded = metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(Value::as_str)?
        .trim()
        .strip_prefix('#')?;
    if !encoded.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let (red, green, blue, alpha) = match encoded.len() {
        6 => (
            u8::from_str_radix(&encoded[0..2], 16).ok()?,
            u8::from_str_radix(&encoded[2..4], 16).ok()?,
            u8::from_str_radix(&encoded[4..6], 16).ok()?,
            u8::MAX,
        ),
        8 => (
            u8::from_str_radix(&encoded[0..2], 16).ok()?,
            u8::from_str_radix(&encoded[2..4], 16).ok()?,
            u8::from_str_radix(&encoded[4..6], 16).ok()?,
            u8::from_str_radix(&encoded[6..8], 16).ok()?,
        ),
        _ => return None,
    };
    Some(UiRgbaColor::from_u8(red, green, blue, alpha))
}

fn positive_float_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    nonnegative_float_attribute(metadata, key).filter(|value| *value > 0.0)
}

fn nonnegative_float_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    match metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))?
    {
        Value::Float(value) if value.is_finite() && *value >= 0.0 => {
            let value = *value as f32;
            value.is_finite().then_some(value)
        }
        Value::Integer(value) if *value >= 0 => {
            let value = *value as f32;
            value.is_finite().then_some(value)
        }
        _ => None,
    }
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
    background: UiRgbaColor,
    border: Option<UiRgbaColor>,
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
            background_color: Some(css_color(background)),
            border_color: border.map(css_color),
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
    foreground: UiRgbaColor,
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
            foreground_color: Some(css_color(foreground)),
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

fn css_color(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    let mut value = if alpha == u8::MAX {
        format!("{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("{red:02x}{green:02x}{blue:02x}{alpha:02x}")
    };
    value.insert(0, '#');
    value
}
