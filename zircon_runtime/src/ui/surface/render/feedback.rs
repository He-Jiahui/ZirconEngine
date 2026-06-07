use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

const TOOLTIP_PADDING_X: f32 = 8.0;
const TOOLTIP_TITLE_TOP: f32 = 7.0;
const TOOLTIP_BODY_TOP: f32 = 23.0;
const TOOLTIP_ICON_SIZE: f32 = 18.0;
const TOOLTIP_TITLE_FONT_SIZE: f32 = 12.0;
const TOOLTIP_TITLE_LINE_HEIGHT: f32 = 14.0;
const TOOLTIP_BODY_FONT_SIZE: f32 = 11.0;
const TOOLTIP_BODY_LINE_HEIGHT: f32 = 13.0;
const ALERT_ICON_LEFT: f32 = 10.0;
const ALERT_ICON_SIZE: f32 = 18.0;
const ALERT_TEXT_GAP: f32 = 8.0;
const ALERT_TEXT_RIGHT_INSET: f32 = 10.0;
const ALERT_ACTION_WIDTH: f32 = 44.0;
const ALERT_FONT_SIZE: f32 = 12.0;
const ALERT_LINE_HEIGHT: f32 = ALERT_FONT_SIZE * 1.2;
const ALERT_TITLE_FONT_SIZE: f32 = 13.0;
const ALERT_TITLE_LINE_HEIGHT: f32 = ALERT_TITLE_FONT_SIZE * 1.2;
const TOAST_ICON_LEFT: f32 = 12.0;
const TOAST_ICON_SIZE: f32 = 18.0;
const TOAST_TEXT_GAP: f32 = 9.0;
const TOAST_TRAILING_INSET: f32 = 10.0;
const TOAST_ACTION_WIDTH: f32 = 44.0;
const TOAST_CLOSE_SIZE: f32 = 14.0;
const TOAST_FONT_SIZE: f32 = 11.5;
const TOAST_LINE_HEIGHT: f32 = TOAST_FONT_SIZE * 1.25;

const TOOLTIP_SURFACE: &str = "#171c20";
const TOOLTIP_BORDER: &str = "#252d32";
const TOOLTIP_TITLE: &str = "#d0d9dd";
const TOOLTIP_BODY: &str = "#a8b3b8";
const TOOLTIP_ICON: &str = "#259ca7";
const ALERT_INFO_SURFACE: &str = "#122e48";
const ALERT_INFO_BORDER: &str = "#296596";
const ALERT_INFO_MARK: &str = "#35c7d0";
const ALERT_SUCCESS_SURFACE: &str = "#163927";
const ALERT_SUCCESS_BORDER: &str = "#357348";
const ALERT_SUCCESS_MARK: &str = "#42b883";
const ALERT_WARNING_SURFACE: &str = "#453214";
const ALERT_WARNING_BORDER: &str = "#845e23";
const ALERT_WARNING_MARK: &str = "#e0a33a";
const ALERT_ERROR_SURFACE: &str = "#482024";
const ALERT_ERROR_BORDER: &str = "#853d3a";
const ALERT_ERROR_MARK: &str = "#ef7066";
const TOAST_SURFACE: &str = "#153035";
const TOAST_SURFACE_HOVER: &str = "#183a3f";
const TOAST_SURFACE_PRESSED: &str = "#103c4a";
const TOAST_BORDER: &str = "#35c7d014";
const TOAST_TEXT: &str = "#cee0e2";
const TOAST_ACTION: &str = "#35c7d0";
const DISABLED_SURFACE: &str = "#252c31";
const DISABLED_BORDER: &str = "#343f47";
const DISABLED_TEXT: &str = "#59656c";
const FOCUS_BORDER: &str = "#35c7d0";

pub(super) fn feedback_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| feedback_kind(metadata).is_some())
}

pub(super) fn feedback_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| feedback_kind(metadata).is_some())
}

pub(super) fn feedback_render_commands(
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
    let Some(kind) = feedback_kind(metadata) else {
        return Vec::new();
    };
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = FeedbackRenderState::resolve(kind, metadata, state_flags, component_state);
    match kind {
        FeedbackKind::Alert => alert_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
        FeedbackKind::AlertTitle => alert_title_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
        FeedbackKind::Tooltip => tooltip_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
        FeedbackKind::Toast => toast_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeedbackKind {
    Alert,
    AlertTitle,
    Tooltip,
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
struct FeedbackRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl FeedbackRenderState {
    fn resolve(
        kind: FeedbackKind,
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        let family = match kind {
            FeedbackKind::Alert | FeedbackKind::AlertTitle => UiPainterFamily::Alert,
            FeedbackKind::Tooltip => UiPainterFamily::Tooltip,
            FeedbackKind::Toast => UiPainterFamily::Toast,
        };
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
        }
    }

    fn disabled(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Disabled)
    }

    fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }

    fn hot(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Focused
                | UiPainterResolvedState::Open
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}

fn feedback_kind(metadata: &UiTemplateNodeMetadata) -> Option<FeedbackKind> {
    match metadata.component.as_str() {
        "Alert" => Some(FeedbackKind::Alert),
        "AlertTitle" => Some(FeedbackKind::AlertTitle),
        "Tooltip" => Some(FeedbackKind::Tooltip),
        "Toast" | "Snackbar" | "SnackbarContent" => Some(FeedbackKind::Toast),
        _ => None,
    }
}

fn alert_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let tone = alert_tone(metadata);
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        alert_surface_color(metadata, state, tone),
        Some(alert_border_color(metadata, state, tone)),
        border_width(metadata),
        corner_radius(metadata, 4.0),
        state,
        opacity,
    )];

    let icon_width = if alert_has_icon(metadata) {
        let icon_size = alert_icon_size(metadata);
        let icon_frame = UiFrame::new(
            frame.x + ALERT_ICON_LEFT,
            frame.y + (frame.height - icon_size).max(0.0) * 0.5,
            icon_size,
            icon_size,
        );
        commands.push(icon_command(
            node_id,
            icon_frame,
            clip_frame,
            z_index.saturating_add(2),
            alert_icon(metadata, tone),
            alert_mark_color(metadata, state, tone),
            state,
            opacity,
        ));
        icon_size + ALERT_TEXT_GAP
    } else {
        0.0
    };

    let action = alert_action(metadata);
    let action_left = frame.right() - ALERT_TEXT_RIGHT_INSET - ALERT_ACTION_WIDTH;
    let text_left = frame.x + ALERT_ICON_LEFT + icon_width;
    let text_right = if action.is_some() {
        action_left - 4.0
    } else {
        frame.right() - ALERT_TEXT_RIGHT_INSET
    };
    if let Some(message) = alert_message(metadata) {
        if text_right > text_left {
            commands.push(text_command(
                node_id,
                UiFrame::new(
                    text_left,
                    frame.y + (frame.height - ALERT_LINE_HEIGHT).max(0.0) * 0.5,
                    text_right - text_left,
                    ALERT_LINE_HEIGHT,
                ),
                clip_frame,
                z_index.saturating_add(3),
                message,
                alert_text_color(metadata, state, tone),
                ALERT_FONT_SIZE,
                ALERT_LINE_HEIGHT,
                state,
                opacity,
            ));
        }
    }
    if let Some(action) = action {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                action_left,
                frame.y + (frame.height - ALERT_LINE_HEIGHT).max(0.0) * 0.5,
                ALERT_ACTION_WIDTH,
                ALERT_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(4),
            action,
            alert_action_color(metadata, state, tone),
            ALERT_FONT_SIZE,
            ALERT_LINE_HEIGHT,
            state,
            opacity,
        ));
    }
    commands
}

fn alert_title_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let Some(title) = alert_title(metadata) else {
        return Vec::new();
    };
    let tone = alert_tone(metadata);
    vec![text_command(
        node_id,
        UiFrame::new(
            frame.x,
            frame.y + (frame.height - ALERT_TITLE_LINE_HEIGHT).max(0.0) * 0.5,
            frame.width,
            ALERT_TITLE_LINE_HEIGHT,
        ),
        clip_frame,
        z_index.saturating_add(2),
        title,
        alert_text_color(metadata, state, tone),
        ALERT_TITLE_FONT_SIZE,
        ALERT_TITLE_LINE_HEIGHT,
        state,
        opacity,
    )]
}

fn tooltip_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        tooltip_surface_color(metadata, state),
        Some(tooltip_border_color(metadata, state)),
        border_width(metadata),
        corner_radius(metadata, 4.0),
        state,
        opacity,
    )];

    let icon = tooltip_icon(metadata);
    let icon_width = icon
        .as_ref()
        .map(|_| tooltip_icon_size(metadata))
        .unwrap_or(0.0);
    let text_width = (frame.width - TOOLTIP_PADDING_X * 2.0 - icon_width - 6.0).max(1.0);
    if let Some(title) = tooltip_title(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + TOOLTIP_PADDING_X,
                frame.y + TOOLTIP_TITLE_TOP,
                text_width,
                TOOLTIP_TITLE_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(3),
            title,
            tooltip_title_color(metadata, state),
            TOOLTIP_TITLE_FONT_SIZE,
            TOOLTIP_TITLE_LINE_HEIGHT,
            state,
            opacity,
        ));
    }
    if let Some(body) = tooltip_body(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + TOOLTIP_PADDING_X,
                frame.y + TOOLTIP_BODY_TOP,
                text_width,
                TOOLTIP_BODY_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(4),
            body,
            tooltip_body_color(metadata, state),
            TOOLTIP_BODY_FONT_SIZE,
            TOOLTIP_BODY_LINE_HEIGHT,
            state,
            opacity,
        ));
    }
    if let Some(icon) = icon {
        let size = tooltip_icon_size(metadata);
        commands.push(icon_command(
            node_id,
            UiFrame::new(
                frame.right() - TOOLTIP_PADDING_X - size,
                frame.y + (frame.height - size).max(0.0) * 0.5,
                size,
                size,
            ),
            clip_frame,
            z_index.saturating_add(5),
            icon,
            tooltip_icon_color(metadata, state),
            state,
            opacity,
        ));
    }
    commands
}

fn toast_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        toast_surface_color(metadata, state),
        Some(toast_border_color(metadata, state)),
        border_width(metadata),
        corner_radius(metadata, 5.0),
        state,
        opacity,
    )];

    let icon_size = toast_icon_size(metadata);
    let icon_frame = UiFrame::new(
        frame.x + TOAST_ICON_LEFT,
        frame.y + (frame.height - icon_size).max(0.0) * 0.5,
        icon_size,
        icon_size,
    );
    commands.push(icon_command(
        node_id,
        icon_frame,
        clip_frame,
        z_index.saturating_add(2),
        toast_icon(metadata),
        toast_mark_color(metadata, state),
        state,
        opacity,
    ));

    let close = toast_close_rect(frame);
    let action = toast_action(metadata);
    let action_left = if action.is_some() {
        close.x - TOAST_ACTION_WIDTH
    } else {
        close.x
    };
    let text_left = icon_frame.x + icon_frame.width + TOAST_TEXT_GAP;
    let text_right = if action.is_some() {
        action_left - 4.0
    } else {
        frame.right() - TOAST_TRAILING_INSET
    };
    if let Some(text) = toast_text(metadata) {
        if text_right > text_left {
            commands.push(text_command(
                node_id,
                UiFrame::new(
                    text_left,
                    frame.y + (frame.height - TOAST_LINE_HEIGHT).max(0.0) * 0.5,
                    text_right - text_left,
                    TOAST_LINE_HEIGHT,
                ),
                clip_frame,
                z_index.saturating_add(3),
                text,
                toast_text_color(metadata, state),
                TOAST_FONT_SIZE,
                TOAST_LINE_HEIGHT,
                state,
                opacity,
            ));
        }
    }
    if let Some(action) = action {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                action_left,
                frame.y + (frame.height - TOAST_LINE_HEIGHT).max(0.0) * 0.5,
                TOAST_ACTION_WIDTH,
                TOAST_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(4),
            action,
            toast_action_color(metadata, state),
            TOAST_FONT_SIZE,
            TOAST_LINE_HEIGHT,
            state,
            opacity,
        ));
    }
    commands
}

fn tooltip_title(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(metadata, &["title", "text"]).or_else(|| Some("Tooltip".to_string()))
}

fn alert_tone(metadata: &UiTemplateNodeMetadata) -> AlertTone {
    let joined = [
        string_attribute(metadata, "severity"),
        string_attribute(metadata, "color"),
        string_attribute(metadata, "validation_level"),
        string_attribute(metadata, "text_tone"),
        string_attribute(metadata, "component_variant"),
        string_attribute(metadata, "icon"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_ascii_lowercase();

    if joined.contains("warning") {
        AlertTone::Warning
    } else if joined.contains("error") || joined.contains("danger") || joined.contains("failed") {
        AlertTone::Error
    } else if joined.contains("info") {
        AlertTone::Info
    } else {
        AlertTone::Success
    }
}

fn alert_has_icon(metadata: &UiTemplateNodeMetadata) -> bool {
    !matches!(metadata.attributes.get("icon"), Some(Value::Boolean(false)))
        && !matches!(
            metadata.attributes.get("show_icon"),
            Some(Value::Boolean(false))
        )
}

fn alert_message(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(
        metadata,
        &[
            "message",
            "text",
            "label",
            "content",
            "description",
            "value_text",
        ],
    )
}

fn alert_title(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(metadata, &["title", "text", "label", "message"])
}

fn alert_action(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(
        metadata,
        &[
            "action",
            "action_text",
            "closeText",
            "close_text",
            "value_action",
        ],
    )
}

fn alert_icon(metadata: &UiTemplateNodeMetadata, tone: AlertTone) -> String {
    first_string(metadata, &["icon", "image", "source"]).unwrap_or_else(|| match tone {
        AlertTone::Info => "info".to_string(),
        AlertTone::Success => "check-circle".to_string(),
        AlertTone::Warning => "alert-triangle".to_string(),
        AlertTone::Error => "x-circle".to_string(),
    })
}

fn tooltip_body(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(
        metadata,
        &["body", "label", "message", "content", "description"],
    )
}

fn tooltip_icon(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(metadata, &["icon", "image", "source"]).or_else(|| Some("info".to_string()))
}

fn toast_text(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(metadata, &["message", "text", "label", "value_text"])
}

fn toast_action(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    first_string(metadata, &["action", "action_text", "value_action"])
}

fn toast_icon(metadata: &UiTemplateNodeMetadata) -> String {
    first_string(metadata, &["icon", "image", "source"]).unwrap_or_else(|| "check-circle".into())
}

fn toast_close_rect(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.right() - TOAST_TRAILING_INSET - TOAST_CLOSE_SIZE,
        frame.y + (frame.height - TOAST_CLOSE_SIZE).max(0.0) * 0.5,
        TOAST_CLOSE_SIZE,
        TOAST_CLOSE_SIZE,
    )
}

fn alert_icon_size(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "layout_icon_size")
        .unwrap_or(ALERT_ICON_SIZE)
        .clamp(10.0, 24.0)
}

fn tooltip_icon_size(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "layout_icon_size")
        .unwrap_or(TOOLTIP_ICON_SIZE)
        .clamp(10.0, 24.0)
}

fn toast_icon_size(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "layout_icon_size")
        .unwrap_or(TOAST_ICON_SIZE)
        .clamp(10.0, 24.0)
}

fn alert_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.disabled() {
        DISABLED_SURFACE
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color")
            .unwrap_or_else(|| alert_tone_surface(tone))
    } else if state.hot() {
        color_attribute(metadata, "hover_background_color").unwrap_or_else(|| {
            color_attribute(metadata, "background_color").unwrap_or(alert_tone_surface(tone))
        })
    } else {
        color_attribute(metadata, "background_color").unwrap_or_else(|| alert_tone_surface(tone))
    }
}

fn alert_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.disabled() {
        DISABLED_BORDER
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color").unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or_else(|| alert_tone_border(tone))
    }
}

fn alert_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "text_color"))
            .unwrap_or_else(|| alert_tone_mark(tone))
    }
}

fn alert_mark_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "icon_color")
            .or_else(|| color_attribute(metadata, "label_color"))
            .or_else(|| color_attribute(metadata, "mark_color"))
            .unwrap_or_else(|| alert_tone_mark(tone))
    }
}

fn alert_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "action_color")
            .or_else(|| color_attribute(metadata, "value_color"))
            .unwrap_or_else(|| alert_text_color(metadata, state, tone))
    }
}

fn alert_tone_surface(tone: AlertTone) -> &'static str {
    match tone {
        AlertTone::Info => ALERT_INFO_SURFACE,
        AlertTone::Success => ALERT_SUCCESS_SURFACE,
        AlertTone::Warning => ALERT_WARNING_SURFACE,
        AlertTone::Error => ALERT_ERROR_SURFACE,
    }
}

fn alert_tone_border(tone: AlertTone) -> &'static str {
    match tone {
        AlertTone::Info => ALERT_INFO_BORDER,
        AlertTone::Success => ALERT_SUCCESS_BORDER,
        AlertTone::Warning => ALERT_WARNING_BORDER,
        AlertTone::Error => ALERT_ERROR_BORDER,
    }
}

fn alert_tone_mark(tone: AlertTone) -> &'static str {
    match tone {
        AlertTone::Info => ALERT_INFO_MARK,
        AlertTone::Success => ALERT_SUCCESS_MARK,
        AlertTone::Warning => ALERT_WARNING_MARK,
        AlertTone::Error => ALERT_ERROR_MARK,
    }
}

fn tooltip_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_SURFACE
    } else {
        color_attribute(metadata, "background_color").unwrap_or(TOOLTIP_SURFACE)
    }
}

fn tooltip_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_BORDER
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color").unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or(TOOLTIP_BORDER)
    }
}

fn tooltip_title_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TOOLTIP_TITLE)
    }
}

fn tooltip_body_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "body_color"))
            .unwrap_or(TOOLTIP_BODY)
    }
}

fn tooltip_icon_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "icon_color").unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "icon_color").unwrap_or(TOOLTIP_ICON)
    }
}

fn toast_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_SURFACE
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color").unwrap_or(TOAST_SURFACE_PRESSED)
    } else if state.hot() {
        color_attribute(metadata, "hover_background_color").unwrap_or(TOAST_SURFACE_HOVER)
    } else {
        color_attribute(metadata, "background_color").unwrap_or(TOAST_SURFACE)
    }
}

fn toast_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_BORDER
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color").unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or(TOAST_BORDER)
    }
}

fn toast_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TOAST_TEXT)
    }
}

fn toast_mark_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "mark_color"))
            .unwrap_or(TOAST_ACTION)
    }
}

fn toast_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "action_color")
            .or_else(|| color_attribute(metadata, "value_color"))
            .unwrap_or(TOAST_ACTION)
    }
}

fn border_width(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "border_width")
        .unwrap_or(1.0)
        .max(0.0)
}

fn corner_radius(metadata: &UiTemplateNodeMetadata, fallback: f32) -> f32 {
    number_attribute(metadata, "corner_radius")
        .or_else(|| number_attribute(metadata, "radius"))
        .unwrap_or(fallback)
        .max(0.0)
}

fn first_string(metadata: &UiTemplateNodeMetadata, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
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
    state: &FeedbackRenderState,
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
        .with_painter_state(state.family, state.visual_state),
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
    state: &FeedbackRenderState,
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
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}

fn icon_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    icon: String,
    foreground: &str,
    state: &FeedbackRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Image,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(foreground.to_string()),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon)),
        opacity,
    }
}
