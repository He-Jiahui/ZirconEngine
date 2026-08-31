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
use super::popup_position::{PopupPlacement, resolve_anchored_popup_geometry};
use crate::ui::text::UiTextMeasureCache;

const DIALOG_PADDING_X: f32 = 20.0;
const DIALOG_TITLE_TOP: f32 = 18.0;
const DIALOG_BODY_TOP: f32 = 48.0;
const DIALOG_ACTION_BOTTOM: f32 = 20.0;
const DIALOG_ACTION_GAP: f32 = 16.0;
const DIALOG_ACTION_MIN_WIDTH: f32 = 56.0;
const DIALOG_ACTION_TEXT_PADDING_X: f32 = 10.0;

#[derive(Clone, Copy)]
struct DialogVisual {
    surface: UiRgbaColor,
    border: UiRgbaColor,
    active_border: UiRgbaColor,
    title: UiRgbaColor,
    body: UiRgbaColor,
    action: UiRgbaColor,
    info: UiRgbaColor,
    info_border: UiRgbaColor,
    warning: UiRgbaColor,
    warning_border: UiRgbaColor,
    error: UiRgbaColor,
    error_border: UiRgbaColor,
    disabled_surface: UiRgbaColor,
    disabled_border: UiRgbaColor,
    disabled_text: UiRgbaColor,
    border_width: f32,
    corner_radius: f32,
    title_font_size: f32,
    title_line_height: f32,
    body_font_size: f32,
    body_line_height: f32,
    action_font_size: f32,
    action_line_height: f32,
}

impl DialogVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_dialog_visual();
        visual.surface =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.surface);
        visual.border = first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.border);
        visual.active_border =
            first_rgba_attribute(metadata, &["focus_border_color"]).unwrap_or(visual.active_border);
        visual.title = first_rgba_attribute(metadata, &["title_color", "foreground_color"])
            .unwrap_or(visual.title);
        visual.body =
            first_rgba_attribute(metadata, &["body_color", "label_color"]).unwrap_or(visual.body);
        visual.action = first_rgba_attribute(metadata, &["action_color"]).unwrap_or(visual.action);
        visual.info = first_rgba_attribute(metadata, &["info_color"]).unwrap_or(visual.info);
        visual.info_border =
            first_rgba_attribute(metadata, &["info_border_color"]).unwrap_or(visual.info_border);
        visual.warning =
            first_rgba_attribute(metadata, &["warning_color"]).unwrap_or(visual.warning);
        visual.warning_border = first_rgba_attribute(metadata, &["warning_border_color"])
            .unwrap_or(visual.warning_border);
        visual.error = first_rgba_attribute(metadata, &["error_color"]).unwrap_or(visual.error);
        visual.error_border =
            first_rgba_attribute(metadata, &["error_border_color"]).unwrap_or(visual.error_border);
        visual.disabled_text = first_rgba_attribute(metadata, &["disabled_text_color"])
            .unwrap_or(visual.disabled_text);
        visual.border_width =
            positive_number_attribute(metadata, "border_width").unwrap_or(visual.border_width);
        visual.corner_radius = nonnegative_number_attribute(metadata, "corner_radius")
            .or_else(|| nonnegative_number_attribute(metadata, "radius"))
            .unwrap_or(visual.corner_radius);
        visual.title_font_size = positive_number_attribute(metadata, "title_font_size")
            .unwrap_or(visual.title_font_size);
        visual.body_font_size =
            positive_number_attribute(metadata, "body_font_size").unwrap_or(visual.body_font_size);
        visual.action_font_size = positive_number_attribute(metadata, "action_font_size")
            .unwrap_or(visual.action_font_size);

        let line_height_ratio = positive_number_attribute(metadata, "typography_line_height_ratio")
            .unwrap_or(visual.title_line_height / visual.title_font_size);
        visual.title_line_height = visual.title_font_size * line_height_ratio;
        visual.body_line_height = visual.body_font_size * line_height_ratio;
        visual.action_line_height = visual.action_font_size * line_height_ratio;
        visual
    }
}

fn default_dialog_visual() -> &'static DialogVisual {
    static VISUAL: OnceLock<DialogVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let palette = &tokens.palette;
        let typography = &tokens.typography;
        let controls = &tokens.controls;

        DialogVisual {
            surface: palette.popup,
            border: palette.border,
            active_border: palette.accent,
            title: palette.text_primary,
            body: palette.text_secondary,
            action: palette.accent,
            info: palette.info,
            info_border: palette.info_container,
            warning: palette.warning,
            warning_border: palette.warning_container,
            error: palette.error,
            error_border: palette.error_container,
            disabled_surface: palette.surface_disabled,
            disabled_border: palette.border_disabled,
            disabled_text: palette.text_disabled,
            border_width: controls.border_width,
            corner_radius: controls.panel_radius,
            title_font_size: typography.title_size,
            title_line_height: typography.title_size * typography.line_height,
            body_font_size: typography.caption_size,
            body_line_height: typography.caption_size * typography.line_height,
            action_font_size: typography.body_size,
            action_line_height: typography.body_size * typography.line_height,
        }
    })
}

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
    popup_anchor_frame: Option<UiFrame>,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    text_measure_cache: &mut UiTextMeasureCache,
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
    let (frame, clip_frame) = resolve_anchored_popup_geometry(
        metadata,
        frame,
        popup_anchor_frame,
        clip_frame,
        PopupPlacement::Center,
        0.0,
    );

    let state = DialogRenderState::resolve(metadata, state_flags, component_state);
    let visual = DialogVisual::resolve(metadata);
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        dialog_surface_color(&visual, state),
        Some(dialog_border_color(metadata, kind, state, &visual)),
        visual.border_width,
        visual.corner_radius,
        state.visual_state,
        opacity,
    )];

    if matches!(kind, DialogKind::ConfirmDialog) {
        commands.push(quad_command(
            node_id,
            UiFrame::new(frame.x, frame.y, 4.0, frame.height),
            clip_frame,
            z_index.saturating_add(2),
            severity_mark_color(metadata, &visual),
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
                visual.title_line_height,
            ),
            clip_frame,
            z_index.saturating_add(3),
            title,
            dialog_title_color(metadata, kind, state, &visual),
            visual.title_font_size,
            visual.title_line_height,
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
                visual.body_line_height,
            ),
            clip_frame,
            z_index.saturating_add(4),
            message,
            dialog_body_color(state, &visual),
            visual.body_font_size,
            visual.body_line_height,
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
        &visual,
        frame,
        clip_frame,
        z_index,
        opacity,
        text_measure_cache,
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
    visual: &DialogVisual,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    text_measure_cache: &mut UiTextMeasureCache,
) {
    let action_y = frame.y + frame.height - DIALOG_ACTION_BOTTOM - visual.action_line_height;
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
        let confirm_width = action_width(&confirm, visual, text_measure_cache);
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
                visual.action_line_height,
            ),
            clip_frame,
            z_index.saturating_add(6),
            confirm,
            confirm_action_color(metadata, state, confirm_enabled, visual),
            visual.action_font_size,
            visual.action_line_height,
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
        let cancel_width = action_width(&cancel, visual, text_measure_cache);
        action_right -= cancel_width;
        commands.push(text_command(
            node_id,
            UiFrame::new(
                action_right,
                action_y,
                cancel_width,
                visual.action_line_height,
            ),
            clip_frame,
            z_index.saturating_add(5),
            cancel,
            cancel_action_color(metadata, state, visual),
            visual.action_font_size,
            visual.action_line_height,
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
        let width = action_width(&action, visual, text_measure_cache);
        commands.push(text_command(
            node_id,
            UiFrame::new(
                action_right - width,
                action_y,
                width,
                visual.action_line_height,
            ),
            clip_frame,
            z_index.saturating_add(5),
            action,
            dialog_action_color(state, visual),
            visual.action_font_size,
            visual.action_line_height,
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

fn dialog_surface_color(visual: &DialogVisual, state: DialogRenderState) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_surface
    } else {
        visual.surface
    }
}

fn dialog_border_color(
    metadata: &UiTemplateNodeMetadata,
    kind: DialogKind,
    state: DialogRenderState,
    visual: &DialogVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_border
    } else if matches!(kind, DialogKind::ConfirmDialog) {
        first_rgba_attribute(metadata, &["border_color"])
            .unwrap_or_else(|| severity_border_color(metadata, visual))
    } else if matches!(
        state.visual_state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Open
    ) {
        visual.active_border
    } else {
        visual.border
    }
}

fn dialog_title_color(
    metadata: &UiTemplateNodeMetadata,
    kind: DialogKind,
    state: DialogRenderState,
    visual: &DialogVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_text
    } else if matches!(kind, DialogKind::ConfirmDialog)
        && (bool_attribute(metadata, "destructive").unwrap_or(false)
            || severity(metadata) == DialogSeverity::Error)
    {
        first_rgba_attribute(metadata, &["title_color"])
            .unwrap_or_else(|| severity_mark_color(metadata, visual))
    } else {
        visual.title
    }
}

fn dialog_body_color(state: DialogRenderState, visual: &DialogVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_text
    } else {
        visual.body
    }
}

fn dialog_action_color(state: DialogRenderState, visual: &DialogVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_text
    } else {
        visual.action
    }
}

fn cancel_action_color(
    metadata: &UiTemplateNodeMetadata,
    state: DialogRenderState,
    visual: &DialogVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_text
    } else {
        first_rgba_attribute(metadata, &["cancel_action_color", "cancel_color"])
            .unwrap_or(visual.body)
    }
}

fn confirm_action_color(
    metadata: &UiTemplateNodeMetadata,
    state: DialogRenderState,
    enabled: bool,
    visual: &DialogVisual,
) -> UiRgbaColor {
    if state.unavailable() || !enabled {
        visual.disabled_text
    } else if bool_attribute(metadata, "destructive").unwrap_or(false) {
        first_rgba_attribute(metadata, &["confirm_action_color", "confirm_color"])
            .unwrap_or(visual.error)
    } else {
        first_rgba_attribute(metadata, &["confirm_action_color", "confirm_color"])
            .unwrap_or(visual.action)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogSeverity {
    Info,
    Warning,
    Error,
}

fn severity(metadata: &UiTemplateNodeMetadata) -> DialogSeverity {
    let value = string_attribute(metadata, "severity")
        .unwrap_or("warning")
        .trim();
    if value.eq_ignore_ascii_case("info") {
        DialogSeverity::Info
    } else if value.eq_ignore_ascii_case("error") {
        DialogSeverity::Error
    } else {
        DialogSeverity::Warning
    }
}

fn severity_mark_color(metadata: &UiTemplateNodeMetadata, visual: &DialogVisual) -> UiRgbaColor {
    match severity(metadata) {
        DialogSeverity::Info => visual.info,
        DialogSeverity::Warning => visual.warning,
        DialogSeverity::Error => visual.error,
    }
}

fn severity_border_color(metadata: &UiTemplateNodeMetadata, visual: &DialogVisual) -> UiRgbaColor {
    match severity(metadata) {
        DialogSeverity::Info => visual.info_border,
        DialogSeverity::Warning => visual.warning_border,
        DialogSeverity::Error => visual.error_border,
    }
}

fn action_width(
    text: &str,
    visual: &DialogVisual,
    text_measure_cache: &mut UiTextMeasureCache,
) -> f32 {
    let style = UiResolvedStyle {
        font_size: visual.action_font_size,
        line_height: visual.action_line_height,
        ..UiResolvedStyle::default()
    };
    let measured = text_measure_cache.measure_text_size(text, &style);
    (measured.width + DIALOG_ACTION_TEXT_PADDING_X * 2.0).max(DIALOG_ACTION_MIN_WIDTH)
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
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(value_as_f32)
}

fn positive_number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    number_attribute(metadata, key).filter(|value| *value > 0.0)
}

fn nonnegative_number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    number_attribute(metadata, key).filter(|value| *value >= 0.0)
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn first_rgba_attribute(metadata: &UiTemplateNodeMetadata, keys: &[&str]) -> Option<UiRgbaColor> {
    keys.iter().find_map(|key| {
        metadata
            .style_overrides
            .get(*key)
            .or_else(|| metadata.attributes.get(*key))
            .and_then(Value::as_str)
            .and_then(parse_css_color)
    })
}

fn parse_css_color(value: &str) -> Option<UiRgbaColor> {
    let encoded = value.trim().strip_prefix('#')?;
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

fn value_as_f32(value: &Value) -> Option<f32> {
    let value = value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .filter(|value| value.is_finite())? as f32;
    value.is_finite().then_some(value)
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
        .with_painter_state(UiPainterFamily::Alert, painter_state),
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
