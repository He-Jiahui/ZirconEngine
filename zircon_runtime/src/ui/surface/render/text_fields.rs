use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiEditableTextState, UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
    widget::UiWidgetBehavior,
};

use super::painter_state::UiRenderPainterStateSource;
use crate::ui::text::layout_text;

const DEFAULT_PADDING_X: f32 = 10.0;
const DEFAULT_PADDING_Y: f32 = 4.0;
const DEFAULT_FONT_SIZE: f32 = 11.0;
const DEFAULT_LINE_HEIGHT: f32 = DEFAULT_FONT_SIZE * 1.2;
const SURFACE_IDLE: &str = "#10161a";
const SURFACE_HOVER: &str = "#151d22";
const SURFACE_PRESSED: &str = "#182a30";
const SURFACE_FOCUSED: &str = "#111a1f";
const SURFACE_DISABLED: &str = "#252c31";
const BORDER_IDLE: &str = "#323f47";
const BORDER_HOVER: &str = "#40515a";
const BORDER_FOCUS: &str = "#35c7d0";
const BORDER_DISABLED: &str = "#343f47";
const TEXT: &str = "#c5d0d5";
const TEXT_PLACEHOLDER: &str = "#68747b";
const TEXT_DISABLED: &str = "#59656c";

pub(super) fn text_field_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_text_field)
}

pub(super) fn text_field_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    state_flags: &UiStateFlags,
    component_state: Option<&UiComponentState>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
    visible_text: Option<&str>,
    editable: Option<&UiEditableTextState>,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if !is_text_field(metadata) || frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = TextFieldRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = vec![surface_command(
        node_id, metadata, &state, frame, clip_frame, z_index, opacity,
    )];
    if visible_text.is_some() || editable.is_some_and(|editable| !editable.text.is_empty()) {
        commands.push(text_command(
            node_id,
            metadata,
            &state,
            frame,
            clip_frame,
            z_index.saturating_add(2),
            opacity,
            base_style,
            visible_text.unwrap_or_default(),
            editable,
        ));
    }
    commands
}

#[derive(Clone, Copy)]
struct TextFieldRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl TextFieldRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        let family = UiPainterFamily::TextField;
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
        }
    }

    fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
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

fn is_text_field(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "InputField" | "TextField" | "LineEdit" | "TextEdit" | "NumberField" | "SearchField"
    ) || metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::TextInput
}

fn surface_command(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &TextFieldRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index: z_index.saturating_add(1),
        style: UiResolvedStyle {
            background_color: Some(surface_color(metadata, state).to_string()),
            border_color: Some(border_color(metadata, state).to_string()),
            border_width: border_width(metadata),
            corner_radius: corner_radius(metadata),
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
    metadata: &UiTemplateNodeMetadata,
    state: &TextFieldRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
    visible_text: &str,
    editable: Option<&UiEditableTextState>,
) -> UiRenderCommand {
    let text_frame = text_frame(metadata, frame);
    let text_clip = clip_frame
        .and_then(|clip| clip.intersection(text_frame))
        .unwrap_or(text_frame);
    let mut style = text_style(metadata, state, base_style, visible_text);
    let mut layout = layout_text(visible_text, &style, text_frame, Some(text_clip));
    if state.focused() && !state.unavailable() {
        layout.editable = editable.cloned();
    }
    style = style.with_painter_state(state.family, state.visual_state);
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame: text_frame,
        clip_frame: Some(text_clip),
        z_index,
        style,
        text_layout: Some(layout),
        text: Some(visible_text.to_string()),
        image: None,
        opacity,
    }
}

fn text_frame(metadata: &UiTemplateNodeMetadata, frame: UiFrame) -> UiFrame {
    let left = number_attribute(metadata, "layout_padding_left").unwrap_or(DEFAULT_PADDING_X);
    let right = number_attribute(metadata, "layout_padding_right").unwrap_or(DEFAULT_PADDING_X);
    let top = number_attribute(metadata, "layout_padding_top").unwrap_or(DEFAULT_PADDING_Y);
    let bottom = number_attribute(metadata, "layout_padding_bottom").unwrap_or(DEFAULT_PADDING_Y);
    UiFrame::new(
        frame.x + left,
        frame.y + top,
        (frame.width - left - right).max(1.0),
        (frame.height - top - bottom).max(1.0),
    )
}

fn text_style(
    metadata: &UiTemplateNodeMetadata,
    state: &TextFieldRenderState,
    base_style: &UiResolvedStyle,
    visible_text: &str,
) -> UiResolvedStyle {
    let mut style = base_style.clone();
    style.background_color = None;
    style.border_color = None;
    style.border_width = 0.0;
    style.corner_radius = 0.0;
    style.font_size = number_attribute(metadata, "font_size").unwrap_or(DEFAULT_FONT_SIZE);
    style.line_height = number_attribute(metadata, "line_height").unwrap_or(DEFAULT_LINE_HEIGHT);
    style.foreground_color = Some(text_color(metadata, state, visible_text).to_string());
    style
}

fn surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &TextFieldRenderState,
) -> &'a str {
    if state.unavailable() {
        SURFACE_DISABLED
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color").unwrap_or(SURFACE_PRESSED)
    } else if state.focused() {
        color_attribute(metadata, "focused_background_color")
            .or_else(|| color_attribute(metadata, "focus_background_color"))
            .or_else(|| color_attribute(metadata, "background_color"))
            .unwrap_or(SURFACE_FOCUSED)
    } else if state.hot() {
        color_attribute(metadata, "hover_background_color").unwrap_or(SURFACE_HOVER)
    } else {
        color_attribute(metadata, "background_color").unwrap_or(SURFACE_IDLE)
    }
}

fn border_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &TextFieldRenderState) -> &'a str {
    if state.unavailable() {
        BORDER_DISABLED
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color").unwrap_or(BORDER_FOCUS)
    } else if state.hot() {
        color_attribute(metadata, "hover_border_color").unwrap_or(BORDER_HOVER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or(BORDER_IDLE)
    }
}

fn text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &TextFieldRenderState,
    visible_text: &str,
) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if is_placeholder_text(metadata, visible_text) {
        color_attribute(metadata, "placeholder_color").unwrap_or(TEXT_PLACEHOLDER)
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TEXT)
    }
}

fn is_placeholder_text(metadata: &UiTemplateNodeMetadata, visible_text: &str) -> bool {
    string_attribute(metadata, "placeholder").is_some_and(|placeholder| {
        !placeholder.is_empty()
            && placeholder == visible_text
            && string_attribute(
                metadata,
                metadata.widget.value_property.as_deref().unwrap_or("value"),
            )
            .unwrap_or_default()
            .is_empty()
    })
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
