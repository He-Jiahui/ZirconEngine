use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiPainterStyleSelector},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

const DEFAULT_PADDING_X: f32 = 12.0;
const DEFAULT_ICON_SIZE: f32 = 16.0;
const DEFAULT_ICON_BUTTON_SIZE: f32 = 18.0;
const DEFAULT_SPACING: f32 = 7.0;
const DEFAULT_RADIUS: f32 = 4.0;
const DEFAULT_FONT_SIZE: f32 = 11.0;
const DEFAULT_LINE_HEIGHT: f32 = DEFAULT_FONT_SIZE * 1.2;

const PRIMARY_SURFACE: &str = "#32b8c5";
const PRIMARY_SURFACE_HOVER: &str = "#43ccd8";
const PRIMARY_SURFACE_PRESSED: &str = "#1e8c99";
const PRIMARY_BORDER: &str = "#249aa6";
const PRIMARY_TEXT: &str = "#08181b";
const SECONDARY_SURFACE: &str = "#191f23";
const SECONDARY_SURFACE_HOVER: &str = "#20282d";
const SECONDARY_SURFACE_PRESSED: &str = "#12343d";
const SECONDARY_BORDER: &str = "#3a464e";
const SECONDARY_TEXT: &str = "#c9d5da";
const TERTIARY_SURFACE: &str = "#15191d";
const TERTIARY_TEXT: &str = "#98a6ae";
const DANGER_SURFACE: &str = "#482024";
const DANGER_BORDER: &str = "#7a3937";
const DANGER_TEXT: &str = "#ef7066";
const DISABLED_SURFACE: &str = "#252c31";
const DISABLED_BORDER: &str = "#343f47";
const DISABLED_TEXT: &str = "#59656c";
const FOCUS_BORDER: &str = "#35c7d0";
const ICON_NORMAL: &str = "#a4aeb4";
const ICON_SELECTED_SURFACE: &str = "#14373c";
const ICON_PANEL_SURFACE: &str = "#1f2529";
const ICON_PANEL_BORDER: &str = "#30393f";

pub(super) fn button_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_button_component)
}

pub(super) fn button_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_button_component)
}

pub(super) fn button_render_commands(
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
    if !is_button_component(metadata) || frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = ButtonRenderState::resolve(metadata, state_flags, component_state);
    if is_icon_button(metadata) {
        icon_button_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        )
    } else {
        button_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        )
    }
}

#[derive(Clone, Copy)]
struct ButtonRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl ButtonRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let component_flags = component_state.map(|state| &state.flags);
        let selected = component_flags.is_some_and(|flags| flags.selected)
            || bool_attribute(metadata, "selected").unwrap_or(false);
        let checked = component_flags.is_some_and(|flags| flags.checked)
            || state_flags.checked
            || bool_attribute(metadata, "checked").unwrap_or(false);
        let mut painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        painter_state.checked = checked;
        painter_state.selected = selected;
        let family = if is_icon_button(metadata) {
            UiPainterFamily::IconButton
        } else {
            UiPainterFamily::Button
        };
        Self {
            family,
            visual_state: UiPainterStyleSelector::resolved_state_for_family(
                painter_state,
                family,
            ),
        }
    }

    fn disabled(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Disabled)
    }

    fn selected(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Selected | UiPainterResolvedState::Checked
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
                | UiPainterResolvedState::Open
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}

fn is_button_component(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "Button" | "ToggleButton" | "IconButton"
    )
}

fn is_icon_button(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "IconButton"
}

fn button_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &ButtonRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = vec![surface_command(
        node_id, frame, clip_frame, z_index, metadata, state, opacity,
    )];
    let icon = icon_name(metadata);
    let label = button_label(metadata);
    let icon_size = number_attribute(metadata, "layout_icon_size").unwrap_or(DEFAULT_ICON_SIZE);
    let gap = number_attribute(metadata, "layout_spacing").unwrap_or(DEFAULT_SPACING);
    let text_width = (frame.width
        - padding_left(metadata)
        - padding_right(metadata)
        - icon.as_ref().map(|_| icon_size + gap).unwrap_or(0.0))
    .max(1.0);
    let mut cursor_x = frame.x + padding_left(metadata);
    if let Some(icon) = icon {
        let icon_frame = UiFrame::new(
            cursor_x,
            frame.y + (frame.height - icon_size).max(0.0) * 0.5,
            icon_size,
            icon_size,
        );
        commands.push(icon_command(
            node_id,
            icon_frame,
            clip_frame,
            z_index.saturating_add(3),
            icon,
            foreground_color(metadata, state),
            state,
            opacity,
        ));
        cursor_x += icon_size + gap;
    }
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                cursor_x,
                frame.y + (frame.height - DEFAULT_LINE_HEIGHT).max(0.0) * 0.5,
                text_width,
                DEFAULT_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(4),
            label,
            foreground_color(metadata, state),
            state,
            opacity,
        ));
    }
    commands
}

fn icon_button_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &ButtonRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = vec![surface_command(
        node_id, frame, clip_frame, z_index, metadata, state, opacity,
    )];
    let Some(icon) = icon_name(metadata) else {
        return commands;
    };
    let size = number_attribute(metadata, "layout_icon_size").unwrap_or(DEFAULT_ICON_BUTTON_SIZE);
    commands.push(icon_command(
        node_id,
        UiFrame::new(
            frame.x + (frame.width - size).max(0.0) * 0.5,
            frame.y + (frame.height - size).max(0.0) * 0.5,
            size,
            size,
        ),
        clip_frame,
        z_index.saturating_add(3),
        icon,
        icon_button_foreground(metadata, state),
        state,
        opacity,
    ));
    commands
}

fn surface_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    metadata: &UiTemplateNodeMetadata,
    state: &ButtonRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index: z_index.saturating_add(1),
        style: UiResolvedStyle {
            background_color: Some(background_color(metadata, state).to_string()),
            border_color: Some(border_color(metadata, state).to_string()),
            border_width: border_width(metadata, state),
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
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: &str,
    state: &ButtonRenderState,
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
            font_size: DEFAULT_FONT_SIZE,
            line_height: DEFAULT_LINE_HEIGHT,
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
    state: &ButtonRenderState,
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

fn background_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ButtonRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_SURFACE
    } else if is_icon_button(metadata) {
        icon_button_background(metadata, state)
    } else if state.pressed() {
        match button_kind(metadata) {
            ButtonKind::Primary => PRIMARY_SURFACE_PRESSED,
            ButtonKind::Tertiary => SECONDARY_SURFACE_PRESSED,
            ButtonKind::Danger => DANGER_SURFACE,
            ButtonKind::Secondary => SECONDARY_SURFACE_PRESSED,
        }
    } else if state.hot() {
        match button_kind(metadata) {
            ButtonKind::Primary => PRIMARY_SURFACE_HOVER,
            ButtonKind::Tertiary => SECONDARY_SURFACE_HOVER,
            ButtonKind::Danger => DANGER_SURFACE,
            ButtonKind::Secondary => SECONDARY_SURFACE_HOVER,
        }
    } else {
        match button_kind(metadata) {
            ButtonKind::Primary => {
                color_attribute(metadata, "background_color").unwrap_or(PRIMARY_SURFACE)
            }
            ButtonKind::Tertiary => {
                color_attribute(metadata, "background_color").unwrap_or(TERTIARY_SURFACE)
            }
            ButtonKind::Danger => {
                color_attribute(metadata, "background_color").unwrap_or(DANGER_SURFACE)
            }
            ButtonKind::Secondary => {
                color_attribute(metadata, "background_color").unwrap_or(SECONDARY_SURFACE)
            }
        }
    }
}

fn icon_button_background<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ButtonRenderState,
) -> &'a str {
    if state.selected() {
        color_attribute(metadata, "selected_background_color").unwrap_or(ICON_SELECTED_SURFACE)
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color").unwrap_or(SECONDARY_SURFACE_PRESSED)
    } else if state.hot() {
        color_attribute(metadata, "hover_background_color").unwrap_or(SECONDARY_SURFACE_HOVER)
    } else {
        color_attribute(metadata, "background_color").unwrap_or(ICON_PANEL_SURFACE)
    }
}

fn border_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &ButtonRenderState) -> &'a str {
    if state.disabled() {
        DISABLED_BORDER
    } else if state.focused() || state.pressed() || state.selected() {
        color_attribute(metadata, "focus_border_color").unwrap_or(FOCUS_BORDER)
    } else if is_icon_button(metadata) {
        color_attribute(metadata, "border_color").unwrap_or(ICON_PANEL_BORDER)
    } else {
        match button_kind(metadata) {
            ButtonKind::Primary => {
                color_attribute(metadata, "border_color").unwrap_or(PRIMARY_BORDER)
            }
            ButtonKind::Danger => {
                color_attribute(metadata, "border_color").unwrap_or(DANGER_BORDER)
            }
            ButtonKind::Secondary | ButtonKind::Tertiary => {
                color_attribute(metadata, "border_color").unwrap_or(SECONDARY_BORDER)
            }
        }
    }
}

fn foreground_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ButtonRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else {
        match button_kind(metadata) {
            ButtonKind::Primary => {
                color_attribute(metadata, "foreground_color").unwrap_or(PRIMARY_TEXT)
            }
            ButtonKind::Danger => {
                color_attribute(metadata, "foreground_color").unwrap_or(DANGER_TEXT)
            }
            ButtonKind::Tertiary => {
                color_attribute(metadata, "foreground_color").unwrap_or(TERTIARY_TEXT)
            }
            ButtonKind::Secondary => {
                color_attribute(metadata, "foreground_color").unwrap_or(SECONDARY_TEXT)
            }
        }
    }
}

fn icon_button_foreground<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ButtonRenderState,
) -> &'a str {
    if state.disabled() {
        DISABLED_TEXT
    } else if state.selected() || state.focused() || state.pressed() {
        color_attribute(metadata, "selected_icon_color")
            .or_else(|| color_attribute(metadata, "icon_color"))
            .unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "icon_color")
            .or_else(|| color_attribute(metadata, "foreground_color"))
            .unwrap_or(ICON_NORMAL)
    }
}

fn border_width(metadata: &UiTemplateNodeMetadata, _state: &ButtonRenderState) -> f32 {
    number_attribute(metadata, "border_width")
        .unwrap_or(1.0)
        .max(0.0)
}

fn corner_radius(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "corner_radius")
        .or_else(|| number_attribute(metadata, "radius"))
        .unwrap_or(if is_icon_button(metadata) {
            6.0
        } else {
            DEFAULT_RADIUS
        })
        .max(0.0)
}

#[derive(Clone, Copy)]
enum ButtonKind {
    Primary,
    Secondary,
    Tertiary,
    Danger,
}

fn button_kind(metadata: &UiTemplateNodeMetadata) -> ButtonKind {
    let joined = [
        string_attribute(metadata, "button_color"),
        string_attribute(metadata, "button_variant"),
        string_attribute(metadata, "validation_level"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_ascii_lowercase();
    if joined.contains("danger") || joined.contains("error") {
        ButtonKind::Danger
    } else if joined.contains("primary") {
        ButtonKind::Primary
    } else if joined.contains("tertiary") || joined.contains("text") {
        ButtonKind::Tertiary
    } else {
        ButtonKind::Secondary
    }
}

fn button_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["text", "label", "value_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

fn icon_name(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["icon", "image", "source"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|icon| !icon.is_empty())
        .map(|icon| {
            icon.rsplit(['/', '\\'])
                .next()
                .unwrap_or(icon)
                .trim_end_matches(".svg")
                .to_string()
        })
}

fn padding_left(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "layout_padding_left").unwrap_or(DEFAULT_PADDING_X)
}

fn padding_right(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "layout_padding_right").unwrap_or(DEFAULT_PADDING_X)
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
