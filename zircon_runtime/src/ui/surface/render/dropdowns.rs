use toml::Value;
use zircon_runtime_interface::ui::{
    component::{UiComponentState, UiValue},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

const HORIZONTAL_INSET: f32 = 8.0;
const CARET_SIZE: f32 = 12.0;
const CARET_RIGHT_INSET: f32 = 12.0;
const OPEN_MARK_WIDTH: f32 = 2.0;
const LABEL_FONT_SIZE: f32 = 10.0;
const LABEL_LINE_HEIGHT: f32 = 12.0;
const VALUE_FONT_SIZE: f32 = 11.0;
const VALUE_LINE_HEIGHT: f32 = VALUE_FONT_SIZE * 1.2;
const SURFACE_IDLE: &str = "#10161a";
const SURFACE_HOVER: &str = "#1a2429";
const SURFACE_PRESSED: &str = "#203239";
const SURFACE_OPEN: &str = "#16282d";
const SURFACE_DISABLED: &str = "#252c31";
const BORDER_IDLE: &str = "#323f47";
const BORDER_FOCUS: &str = "#35c7d0";
const BORDER_DISABLED: &str = "#343f47";
const TEXT: &str = "#c5d0d5";
const LABEL_TEXT: &str = "#7f8c94";
const TEXT_DISABLED: &str = "#59656c";

pub(super) fn dropdown_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_dropdown)
}

pub(super) fn dropdown_render_commands(
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
    if !is_dropdown(metadata) || frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = DropdownRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = Vec::new();
    commands.push(quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        surface_color(metadata, &state),
        Some(border_color(metadata, &state)),
        border_width(metadata),
        corner_radius(metadata),
        &state,
        opacity,
    ));

    let caret = caret_rect(frame);
    let text_right = (caret.x - 4.0).max(frame.x + HORIZONTAL_INSET + 1.0);
    if let Some(label) = dropdown_label(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + HORIZONTAL_INSET,
                frame.y + 4.0,
                text_right - frame.x - HORIZONTAL_INSET,
                LABEL_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(3),
            label,
            label_color(metadata, &state),
            LABEL_FONT_SIZE,
            LABEL_LINE_HEIGHT,
            &state,
            opacity,
        ));
    }
    if let Some(value) = selected_value_text(metadata) {
        commands.push(text_command(
            node_id,
            value_rect(frame, text_right, dropdown_label(metadata).is_some()),
            clip_frame,
            z_index.saturating_add(3),
            value,
            text_color(metadata, &state),
            VALUE_FONT_SIZE,
            VALUE_LINE_HEIGHT,
            &state,
            opacity,
        ));
    }

    commands.push(icon_command(
        node_id,
        caret,
        clip_frame,
        z_index.saturating_add(4),
        if state.open() {
            "chevron-up"
        } else {
            "chevron-down"
        },
        icon_color(metadata, &state),
        &state,
        opacity,
    ));
    if state.open() {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                (frame.right() - OPEN_MARK_WIDTH - 2.0).max(frame.x),
                frame.y + 5.0,
                OPEN_MARK_WIDTH,
                (frame.height - 10.0).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(5),
            border_color(metadata, &state),
            None,
            0.0,
            1.0,
            &state,
            opacity,
        ));
    }
    commands
}

#[derive(Clone, Copy)]
struct DropdownRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl DropdownRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        let family = UiPainterFamily::Dropdown;
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
        }
    }

    fn disabled(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Disabled)
    }

    fn open(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Open)
    }

    fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    fn hot(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Focused
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}

fn is_dropdown(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "ComboBox" | "Dropdown" | "Select"
    )
}

fn value_rect(frame: UiFrame, text_right: f32, has_label: bool) -> UiFrame {
    if has_label && frame.height >= 28.0 {
        UiFrame::new(
            frame.x + HORIZONTAL_INSET,
            (frame.y + frame.height - VALUE_LINE_HEIGHT - 2.0).round(),
            text_right - frame.x - HORIZONTAL_INSET,
            VALUE_LINE_HEIGHT,
        )
    } else {
        UiFrame::new(
            frame.x + HORIZONTAL_INSET,
            (frame.y + (frame.height - VALUE_LINE_HEIGHT).max(0.0) * 0.5).round(),
            text_right - frame.x - HORIZONTAL_INSET,
            VALUE_LINE_HEIGHT,
        )
    }
}

fn caret_rect(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x + frame.width - CARET_RIGHT_INSET - CARET_SIZE,
        frame.y + (frame.height - CARET_SIZE).max(0.0) * 0.5,
        CARET_SIZE,
        CARET_SIZE,
    )
}

fn selected_value_text(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    string_attribute(metadata, "value_text")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| option_label_for_value(metadata))
        .or_else(|| {
            metadata
                .attributes
                .get("value")
                .map(UiValue::from_toml)
                .map(|value| value.display_text())
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            string_attribute(metadata, "placeholder")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

fn option_label_for_value(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    let selected = metadata.attributes.get("value")?;
    let selected = UiValue::from_toml(selected).display_text();
    if selected.is_empty() {
        return None;
    }
    metadata
        .attributes
        .get("options")
        .and_then(Value::as_array)
        .and_then(|options| {
            options.iter().find_map(|option| {
                let (id, label) = option_id_and_label(option)?;
                (id == selected || label == selected).then_some(label)
            })
        })
}

fn option_id_and_label(value: &Value) -> Option<(String, String)> {
    match value {
        Value::String(raw) => {
            let mut parts = raw.splitn(3, '|');
            let id = parts.next().unwrap_or_default().trim().to_string();
            let flags = parts.next().unwrap_or_default();
            let label = flag_value(flags, "label")
                .or_else(|| flag_value(flags, "text"))
                .unwrap_or_else(|| id.clone());
            Some((id, label))
        }
        Value::Table(table) => {
            let id = table
                .get("id")
                .or_else(|| table.get("value"))
                .or_else(|| table.get("label"))
                .or_else(|| table.get("text"))
                .and_then(Value::as_str)?
                .trim()
                .to_string();
            let label = table
                .get("label")
                .or_else(|| table.get("text"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|label| !label.is_empty())
                .unwrap_or(id.as_str())
                .to_string();
            Some((id, label))
        }
        _ => None,
    }
}

fn flag_value(flags: &str, expected_key: &str) -> Option<String> {
    flags.split(',').find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case(expected_key)
            .then(|| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn dropdown_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "label_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

fn surface_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &DropdownRenderState) -> &'a str {
    if state.disabled() {
        SURFACE_DISABLED
    } else if state.open() {
        color_attribute(metadata, "open_background_color").unwrap_or(SURFACE_OPEN)
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color").unwrap_or(SURFACE_PRESSED)
    } else if state.hot() {
        color_attribute(metadata, "hover_background_color").unwrap_or(SURFACE_HOVER)
    } else {
        color_attribute(metadata, "background_color").unwrap_or(SURFACE_IDLE)
    }
}

fn border_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &DropdownRenderState) -> &'a str {
    if state.disabled() {
        BORDER_DISABLED
    } else if state.open() || state.hot() || state.pressed() {
        color_attribute(metadata, "focus_border_color").unwrap_or(BORDER_FOCUS)
    } else {
        color_attribute(metadata, "border_color").unwrap_or(BORDER_IDLE)
    }
}

fn label_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &DropdownRenderState) -> &'a str {
    if state.disabled() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "label_color").unwrap_or(LABEL_TEXT)
    }
}

fn text_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &DropdownRenderState) -> &'a str {
    if state.disabled() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TEXT)
    }
}

fn icon_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &DropdownRenderState) -> &'a str {
    color_attribute(metadata, "icon_color").unwrap_or_else(|| text_color(metadata, state))
}

fn border_width(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "border_width")
        .unwrap_or(1.0)
        .max(0.0)
}

fn corner_radius(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "corner_radius")
        .or_else(|| number_attribute(metadata, "radius"))
        .unwrap_or(4.0)
        .max(0.0)
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
    state: &DropdownRenderState,
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
    state: &DropdownRenderState,
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
    icon: &str,
    foreground: &str,
    state: &DropdownRenderState,
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
        image: Some(UiVisualAssetRef::Icon(icon.to_string())),
        opacity,
    }
}
