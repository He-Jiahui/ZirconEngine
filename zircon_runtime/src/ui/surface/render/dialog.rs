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

const DIALOG_PADDING_X: f32 = 20.0;
const DIALOG_TITLE_TOP: f32 = 18.0;
const DIALOG_BODY_TOP: f32 = 48.0;
const DIALOG_ACTION_BOTTOM: f32 = 20.0;
const DIALOG_ACTION_GAP: f32 = 16.0;
const DIALOG_ACTION_MIN_WIDTH: f32 = 56.0;
const DIALOG_ACTION_CHAR_WIDTH: f32 = 7.0;
const DIALOG_TITLE_FONT_SIZE: f32 = 15.0;
const DIALOG_TITLE_LINE_HEIGHT: f32 = 18.0;
const DIALOG_BODY_FONT_SIZE: f32 = 12.5;
const DIALOG_BODY_LINE_HEIGHT: f32 = 16.0;
const DIALOG_ACTION_FONT_SIZE: f32 = 12.5;
const DIALOG_ACTION_LINE_HEIGHT: f32 = 16.0;

const DIALOG_SURFACE: &str = "#171c20";
const DIALOG_BORDER: &str = "#343f47";
const DIALOG_ACTIVE_BORDER: &str = "#35c7d0";
const DIALOG_TITLE: &str = "#e8ecee";
const DIALOG_BODY: &str = "#a4aeb4";
const DIALOG_ACTION: &str = "#35c7d0";
const DIALOG_INFO: &str = "#35c7d0";
const DIALOG_INFO_BORDER: &str = "#296596";
const DIALOG_WARNING: &str = "#e0a33a";
const DIALOG_WARNING_BORDER: &str = "#845e23";
const DIALOG_ERROR: &str = "#ef7066";
const DIALOG_ERROR_BORDER: &str = "#853d3a";
const DIALOG_DISABLED_SURFACE: &str = "#252c31";
const DIALOG_DISABLED_BORDER: &str = "#343f47";
const DIALOG_DISABLED_TEXT: &str = "#59656c";

pub(super) fn dialog_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| dialog_kind(metadata).is_some())
}

pub(super) fn dialog_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| dialog_kind(metadata).is_some())
}

pub(super) fn dialog_suppresses_owner_surface(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| dialog_kind(metadata).is_some())
}

pub(super) fn dialog_render_commands(
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
    let Some(kind) = dialog_kind(metadata) else {
        return Vec::new();
    };
    if frame.width <= 1.0 || frame.height <= 1.0 || !dialog_open(metadata) {
        return Vec::new();
    }

    let state = DialogRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        dialog_surface_color(metadata, state),
        Some(dialog_border_color(metadata, kind, state)),
        border_width(metadata),
        corner_radius(metadata),
        state.visual_state,
        opacity,
    )];

    if matches!(kind, DialogKind::ConfirmDialog) {
        commands.push(quad_command(
            node_id,
            UiFrame::new(frame.x, frame.y, 4.0, frame.height),
            clip_frame,
            z_index.saturating_add(2),
            severity_mark_color(metadata),
            None,
            0.0,
            0.0,
            state.visual_state,
            opacity,
        ));
    }

    let content_left = frame.x + DIALOG_PADDING_X;
    let content_width = (frame.width - DIALOG_PADDING_X * 2.0).max(1.0);
    if let Some(title) = dialog_title(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                content_left,
                frame.y + DIALOG_TITLE_TOP,
                content_width,
                DIALOG_TITLE_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(3),
            title,
            dialog_title_color(metadata, kind, state),
            DIALOG_TITLE_FONT_SIZE,
            DIALOG_TITLE_LINE_HEIGHT,
            state.visual_state,
            opacity,
        ));
    }
    if let Some(message) = dialog_message(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                content_left,
                frame.y + DIALOG_BODY_TOP,
                content_width,
                DIALOG_BODY_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(4),
            message,
            dialog_body_color(metadata, state),
            DIALOG_BODY_FONT_SIZE,
            DIALOG_BODY_LINE_HEIGHT,
            state.visual_state,
            opacity,
        ));
    }
    push_dialog_actions(
        &mut commands,
        node_id,
        metadata,
        kind,
        state,
        frame,
        clip_frame,
        z_index,
        opacity,
    );

    commands
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogKind {
    Dialog,
    ConfirmDialog,
}

#[derive(Clone, Copy)]
struct DialogRenderState {
    visual_state: UiPainterResolvedState,
}

impl DialogRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        Self {
            visual_state: painter_state.resolved_state_for_family(UiPainterFamily::Alert),
        }
    }

    fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }
}

fn dialog_kind(metadata: &UiTemplateNodeMetadata) -> Option<DialogKind> {
    match metadata.component.as_str() {
        "Dialog" => Some(DialogKind::Dialog),
        "ConfirmDialog" => Some(DialogKind::ConfirmDialog),
        _ => None,
    }
}

fn push_dialog_actions(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    kind: DialogKind,
    state: DialogRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) {
    let action_y = frame.y + frame.height - DIALOG_ACTION_BOTTOM - DIALOG_ACTION_LINE_HEIGHT;
    let mut action_right = frame.x + frame.width - DIALOG_PADDING_X;
    if matches!(kind, DialogKind::ConfirmDialog) {
        let confirm = first_string(
            metadata,
            &[
                "confirm_text",
                "confirmText",
                "primary_action_text",
                "action",
            ],
        )
        .unwrap_or_else(|| "Confirm".to_string());
        let confirm_width = action_width(&confirm);
        let confirm_enabled = bool_attribute(metadata, "confirm_enabled")
            .or_else(|| bool_attribute(metadata, "confirmEnabled"))
            .unwrap_or(true);
        action_right -= confirm_width;
        commands.push(text_command(
            node_id,
            UiFrame::new(
                action_right,
                action_y,
                confirm_width,
                DIALOG_ACTION_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(6),
            confirm,
            confirm_action_color(metadata, state, confirm_enabled),
            DIALOG_ACTION_FONT_SIZE,
            DIALOG_ACTION_LINE_HEIGHT,
            if confirm_enabled {
                state.visual_state
            } else {
                UiPainterResolvedState::Disabled
            },
            opacity,
        ));
        action_right -= DIALOG_ACTION_GAP;

        let cancel = first_string(metadata, &["cancel_text", "cancelText", "close_text"])
            .unwrap_or_else(|| "Cancel".to_string());
        let cancel_width = action_width(&cancel);
        action_right -= cancel_width;
        commands.push(text_command(
            node_id,
            UiFrame::new(
                action_right,
                action_y,
                cancel_width,
                DIALOG_ACTION_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(5),
            cancel,
            cancel_action_color(metadata, state),
            DIALOG_ACTION_FONT_SIZE,
            DIALOG_ACTION_LINE_HEIGHT,
            state.visual_state,
            opacity,
        ));
        return;
    }

    if let Some(action) = first_string(
        metadata,
        &[
            "action",
            "primary_action_text",
            "confirm_text",
            "close_text",
        ],
    ) {
        let width = action_width(&action);
        commands.push(text_command(
            node_id,
            UiFrame::new(
                action_right - width,
                action_y,
                width,
                DIALOG_ACTION_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(5),
            action,
            dialog_action_color(metadata, state),
            DIALOG_ACTION_FONT_SIZE,
            DIALOG_ACTION_LINE_HEIGHT,
            state.visual_state,
            opacity,
        ));
    }
}

fn dialog_open(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "open")
        .or_else(|| bool_attribute(metadata, "popup_open"))
        .or_else(|| bool_attribute(metadata, "popupOpen"))
        .unwrap_or(false)
}

fn dialog_title(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(metadata, &["title", "text", "label"])
}

fn dialog_message(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(metadata, &["message", "description", "body"])
}

fn dialog_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: DialogRenderState,
) -> &'a str {
    if state.unavailable() {
        DIALOG_DISABLED_SURFACE
    } else {
        color_attribute(metadata, "background_color").unwrap_or(DIALOG_SURFACE)
    }
}

fn dialog_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    kind: DialogKind,
    state: DialogRenderState,
) -> &'a str {
    if state.unavailable() {
        DIALOG_DISABLED_BORDER
    } else if matches!(kind, DialogKind::ConfirmDialog) {
        color_attribute(metadata, "border_color").unwrap_or_else(|| severity_border_color(metadata))
    } else if matches!(
        state.visual_state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Open
    ) {
        color_attribute(metadata, "focus_border_color").unwrap_or(DIALOG_ACTIVE_BORDER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or(DIALOG_BORDER)
    }
}

fn dialog_title_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    kind: DialogKind,
    state: DialogRenderState,
) -> &'a str {
    if state.unavailable() {
        DIALOG_DISABLED_TEXT
    } else if matches!(kind, DialogKind::ConfirmDialog)
        && (bool_attribute(metadata, "destructive").unwrap_or(false)
            || severity(metadata) == DialogSeverity::Error)
    {
        color_attribute(metadata, "title_color").unwrap_or_else(|| severity_mark_color(metadata))
    } else {
        color_attribute(metadata, "title_color")
            .or_else(|| color_attribute(metadata, "foreground_color"))
            .unwrap_or(DIALOG_TITLE)
    }
}

fn dialog_body_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: DialogRenderState,
) -> &'a str {
    if state.unavailable() {
        DIALOG_DISABLED_TEXT
    } else {
        color_attribute(metadata, "body_color")
            .or_else(|| color_attribute(metadata, "label_color"))
            .unwrap_or(DIALOG_BODY)
    }
}

fn dialog_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: DialogRenderState,
) -> &'a str {
    if state.unavailable() {
        DIALOG_DISABLED_TEXT
    } else {
        color_attribute(metadata, "action_color").unwrap_or(DIALOG_ACTION)
    }
}

fn cancel_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: DialogRenderState,
) -> &'a str {
    if state.unavailable() {
        DIALOG_DISABLED_TEXT
    } else {
        color_attribute(metadata, "cancel_action_color")
            .or_else(|| color_attribute(metadata, "cancel_color"))
            .unwrap_or(DIALOG_BODY)
    }
}

fn confirm_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: DialogRenderState,
    enabled: bool,
) -> &'a str {
    if state.unavailable() || !enabled {
        DIALOG_DISABLED_TEXT
    } else if bool_attribute(metadata, "destructive").unwrap_or(false) {
        color_attribute(metadata, "confirm_action_color")
            .or_else(|| color_attribute(metadata, "confirm_color"))
            .unwrap_or(DIALOG_ERROR)
    } else {
        color_attribute(metadata, "confirm_action_color")
            .or_else(|| color_attribute(metadata, "action_color"))
            .unwrap_or(DIALOG_ACTION)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogSeverity {
    Info,
    Warning,
    Error,
}

fn severity(metadata: &UiTemplateNodeMetadata) -> DialogSeverity {
    match string_attribute(metadata, "severity")
        .unwrap_or("warning")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "info" => DialogSeverity::Info,
        "error" => DialogSeverity::Error,
        _ => DialogSeverity::Warning,
    }
}

fn severity_mark_color(metadata: &UiTemplateNodeMetadata) -> &'static str {
    match severity(metadata) {
        DialogSeverity::Info => DIALOG_INFO,
        DialogSeverity::Warning => DIALOG_WARNING,
        DialogSeverity::Error => DIALOG_ERROR,
    }
}

fn severity_border_color(metadata: &UiTemplateNodeMetadata) -> &'static str {
    match severity(metadata) {
        DialogSeverity::Info => DIALOG_INFO_BORDER,
        DialogSeverity::Warning => DIALOG_WARNING_BORDER,
        DialogSeverity::Error => DIALOG_ERROR_BORDER,
    }
}

fn action_width(text: &str) -> f32 {
    (text.chars().count() as f32 * DIALOG_ACTION_CHAR_WIDTH + 20.0).max(DIALOG_ACTION_MIN_WIDTH)
}

fn border_width(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "border_width")
        .unwrap_or(1.0)
        .max(0.0)
}

fn corner_radius(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "corner_radius")
        .or_else(|| number_attribute(metadata, "radius"))
        .unwrap_or(6.0)
        .max(0.0)
}

fn first_string(metadata: &UiTemplateNodeMetadata, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(value_as_f32)
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn color_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(Value::as_str)
        .filter(|color| !color.trim().is_empty())
}

fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
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
        .with_painter_state(UiPainterFamily::Alert, painter_state),
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
        .with_painter_state(UiPainterFamily::Alert, painter_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}
