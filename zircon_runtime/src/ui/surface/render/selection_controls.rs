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

const MARK_INSET_X: f32 = 10.0;
const MARK_SIZE: f32 = 16.0;
const LABEL_GAP: f32 = 9.0;
const LABEL_INSET_Y: f32 = 5.0;
const LABEL_FONT_SIZE: f32 = 11.0;
const LABEL_LINE_HEIGHT: f32 = LABEL_FONT_SIZE * 1.2;
const LABEL_MUTED: &str = "#828c93";
const LABEL_DISABLED: &str = "#59656c";
const MARK_IDLE_FILL: &str = "#141a1e";
const MARK_IDLE_BORDER: &str = "#424e56";
const MARK_DISABLED_FILL: &str = "#252c31";
const MARK_DISABLED_BORDER: &str = "#343f47";
const SURFACE_SELECTED: &str = "#173942";
const ACCENT: &str = "#2aa6b8";
const CHECK_TICK: &str = "#2aa6b8";
const RADIO_CHECKED_FILL: &str = "#1b272d";
const RADIO_CHECKED_BORDER: &str = "#4c5b63";
const RADIO_DOT_SIZE: f32 = 7.0;
const TOGGLE_TRACK_WIDTH: f32 = 34.0;
const TOGGLE_TRACK_HEIGHT: f32 = 18.0;
const TOGGLE_THUMB_SIZE: f32 = 12.0;
const TOGGLE_RIGHT_INSET: f32 = 8.0;
const TOGGLE_THUMB_INSET: f32 = 2.0;
const TOGGLE_TRACK_IDLE: &str = "#232d33";
const TOGGLE_THUMB_IDLE: &str = "#7c878e";
const TOGGLE_BORDER_ON: &str = "#414b54";
const TOGGLE_THUMB_ON: &str = "#a4aeb4";
const TOGGLE_HOVER: &str = "#1a2429";
const TOGGLE_PRESSED: &str = "#223139";
const BORDER_FOCUS: &str = "#35c7d0";

pub(super) fn selection_control_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata
        .and_then(selection_control_kind)
        .is_some_and(|kind| {
            matches!(
                kind,
                SelectionControlKind::Checkbox
                    | SelectionControlKind::Radio
                    | SelectionControlKind::Toggle
            )
        })
}

pub(super) fn selection_control_render_commands(
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
    let Some(kind) = selection_control_kind(metadata) else {
        return Vec::new();
    };
    let state = SelectionRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = Vec::new();
    match kind {
        SelectionControlKind::Checkbox => {
            push_checkbox(
                &mut commands,
                node_id,
                metadata,
                &state,
                frame,
                clip_frame,
                z_index,
                opacity,
            );
        }
        SelectionControlKind::Radio => {
            push_radio(
                &mut commands,
                node_id,
                metadata,
                &state,
                frame,
                clip_frame,
                z_index,
                opacity,
            );
        }
        SelectionControlKind::Toggle => {
            push_toggle(
                &mut commands,
                node_id,
                metadata,
                &state,
                frame,
                clip_frame,
                z_index,
                opacity,
            );
        }
    }
    commands
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionControlKind {
    Checkbox,
    Radio,
    Toggle,
}

#[derive(Clone, Copy)]
struct SelectionRenderState {
    family: UiPainterFamily,
    checked: bool,
    selected: bool,
    visual_state: UiPainterResolvedState,
    surface_hot: bool,
}

impl SelectionRenderState {
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
            || selected
            || bool_attribute(metadata, "checked")
                .or_else(|| bool_attribute(metadata, "value"))
                .unwrap_or(false);
        let disabled = component_flags.is_some_and(|flags| flags.disabled)
            || !state_flags.enabled
            || bool_attribute(metadata, "disabled").unwrap_or(false);
        let mut painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state_with_value_checked();
        painter_state.disabled = disabled;
        painter_state.checked = checked;
        painter_state.selected = selected;
        let family = selection_painter_family(metadata);
        let surface_hot =
            painter_state.hovered || painter_state.dragging || painter_state.drop_hovered;
        Self {
            family,
            checked,
            selected,
            visual_state: painter_state.resolved_state_for_family(family),
            surface_hot,
        }
    }

    fn active(self) -> bool {
        self.checked || self.selected
    }

    fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }

    fn surface_hot(self) -> bool {
        self.surface_hot
    }

    fn focus_border(self) -> bool {
        self.pressed() || self.focused() || (!self.active() && self.surface_hot())
    }
}

fn selection_painter_family(metadata: &UiTemplateNodeMetadata) -> UiPainterFamily {
    match metadata.component.as_str() {
        "Checkbox" => UiPainterFamily::Checkbox,
        "Radio" => UiPainterFamily::Radio,
        "Toggle" | "Switch" => UiPainterFamily::Toggle,
        _ => UiPainterFamily::Generic,
    }
}

fn selection_control_kind(metadata: &UiTemplateNodeMetadata) -> Option<SelectionControlKind> {
    match metadata.component.as_str() {
        "Checkbox" => Some(SelectionControlKind::Checkbox),
        "Radio" => Some(SelectionControlKind::Radio),
        "Toggle" | "Switch" => Some(SelectionControlKind::Toggle),
        _ => None,
    }
}

fn push_checkbox(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) {
    let mark = leading_mark_rect(metadata, frame);
    commands.push(quad_command(
        node_id,
        mark,
        clip_frame,
        z_index.saturating_add(1),
        checkbox_background(metadata, state),
        Some(checkbox_border_color(metadata, state)),
        1.0,
        3.0,
        state,
        opacity,
    ));
    if state.active() {
        push_checkbox_tick(
            commands,
            node_id,
            mark,
            clip_frame,
            z_index.saturating_add(2),
            state,
            opacity,
        );
    }
    push_selection_label(
        commands,
        node_id,
        metadata,
        label_rect_after_mark(metadata, frame, mark),
        clip_frame,
        z_index.saturating_add(4),
        state,
        opacity,
    );
}

fn push_radio(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) {
    let mark = leading_mark_rect(metadata, frame);
    commands.push(quad_command(
        node_id,
        mark,
        clip_frame,
        z_index.saturating_add(1),
        radio_background(metadata, state),
        Some(radio_border_color(metadata, state)),
        1.0,
        mark.height * 0.5,
        state,
        opacity,
    ));
    if state.active() {
        let dot_size = number_attribute(metadata, "dot_size").unwrap_or(RADIO_DOT_SIZE);
        commands.push(quad_command(
            node_id,
            centered_square(mark, dot_size),
            clip_frame,
            z_index.saturating_add(2),
            radio_dot_color(metadata, state),
            None,
            0.0,
            dot_size * 0.5,
            state,
            opacity,
        ));
    }
    push_selection_label(
        commands,
        node_id,
        metadata,
        label_rect_after_mark(metadata, frame, mark),
        clip_frame,
        z_index.saturating_add(4),
        state,
        opacity,
    );
}

fn push_toggle(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) {
    let track = toggle_track_rect(metadata, frame);
    push_selection_label(
        commands,
        node_id,
        metadata,
        toggle_label_rect(metadata, frame, track),
        clip_frame,
        z_index.saturating_add(3),
        state,
        opacity,
    );
    commands.push(quad_command(
        node_id,
        track,
        clip_frame,
        z_index.saturating_add(1),
        toggle_track_color(metadata, state),
        Some(toggle_border_color(metadata, state)),
        1.0,
        track.height * 0.5,
        state,
        opacity,
    ));
    let thumb = toggle_thumb_rect(metadata, state, track);
    commands.push(quad_command(
        node_id,
        thumb,
        clip_frame,
        z_index.saturating_add(2),
        toggle_thumb_color(metadata, state),
        None,
        0.0,
        thumb.height * 0.5,
        state,
        opacity,
    ));
}

fn push_selection_label(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    state: &SelectionRenderState,
    opacity: f32,
) {
    let Some(label) = control_label(metadata) else {
        return;
    };
    if frame.width <= 0.5 || frame.height <= 0.5 {
        return;
    }
    commands.push(text_command(
        node_id,
        frame,
        clip_frame,
        z_index,
        label,
        label_color(metadata, state),
        state,
        opacity,
    ));
}

fn push_checkbox_tick(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    mark: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    state: &SelectionRenderState,
    opacity: f32,
) {
    for tick in [
        UiFrame::new(mark.x + 3.0, mark.y + 7.0, 3.0, 3.0),
        UiFrame::new(mark.x + 5.0, mark.y + 9.0, 3.0, 3.0),
        UiFrame::new(mark.x + 8.0, mark.y + 4.0, 3.0, 8.0),
    ] {
        commands.push(quad_command(
            node_id, tick, clip_frame, z_index, CHECK_TICK, None, 0.0, 1.0, state, opacity,
        ));
    }
}

fn leading_mark_rect(metadata: &UiTemplateNodeMetadata, frame: UiFrame) -> UiFrame {
    let size = number_attribute(metadata, "layout_icon_size").unwrap_or(MARK_SIZE);
    UiFrame::new(
        frame.x + MARK_INSET_X,
        frame.y + (frame.height - size).max(0.0) * 0.5,
        size,
        size,
    )
}

fn label_rect_after_mark(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    mark: UiFrame,
) -> UiFrame {
    let x = mark.x + mark.width + label_gap(metadata);
    UiFrame::new(
        x,
        frame.y + LABEL_INSET_Y,
        (frame.x + frame.width - x - MARK_INSET_X).max(1.0),
        (frame.height - LABEL_INSET_Y * 2.0).max(LABEL_LINE_HEIGHT),
    )
}

fn toggle_label_rect(metadata: &UiTemplateNodeMetadata, frame: UiFrame, track: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x + MARK_INSET_X,
        frame.y + LABEL_INSET_Y,
        (track.x - frame.x - MARK_INSET_X - label_gap(metadata)).max(1.0),
        (frame.height - LABEL_INSET_Y * 2.0).max(LABEL_LINE_HEIGHT),
    )
}

fn toggle_track_rect(metadata: &UiTemplateNodeMetadata, frame: UiFrame) -> UiFrame {
    let width = number_attribute(metadata, "track_width")
        .unwrap_or(TOGGLE_TRACK_WIDTH)
        .min((frame.width - MARK_INSET_X * 2.0).max(1.0));
    let height = number_attribute(metadata, "track_height")
        .unwrap_or(TOGGLE_TRACK_HEIGHT)
        .min(frame.height.max(1.0));
    UiFrame::new(
        (frame.x + frame.width - TOGGLE_RIGHT_INSET - width).max(frame.x),
        frame.y + (frame.height - height).max(0.0) * 0.5,
        width,
        height,
    )
}

fn toggle_thumb_rect(
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    track: UiFrame,
) -> UiFrame {
    let size = number_attribute(metadata, "thumb_size")
        .unwrap_or(TOGGLE_THUMB_SIZE)
        .min(track.width)
        .min(track.height)
        .max(1.0);
    let available = (track.width - size - TOGGLE_THUMB_INSET * 2.0).max(0.0);
    let offset = if state.active() { available } else { 0.0 };
    UiFrame::new(
        track.x + TOGGLE_THUMB_INSET + offset,
        track.y + (track.height - size).max(0.0) * 0.5,
        size,
        size,
    )
}

fn centered_square(frame: UiFrame, size: f32) -> UiFrame {
    let size = size.min(frame.width).min(frame.height).max(1.0);
    UiFrame::new(
        frame.x + (frame.width - size).max(0.0) * 0.5,
        frame.y + (frame.height - size).max(0.0) * 0.5,
        size,
        size,
    )
}

fn checkbox_background<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SelectionRenderState,
) -> &'a str {
    if state.unavailable() {
        MARK_DISABLED_FILL
    } else if state.active() {
        SURFACE_SELECTED
    } else {
        color_attribute(metadata, "background_color").unwrap_or(MARK_IDLE_FILL)
    }
}

fn checkbox_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SelectionRenderState,
) -> &'a str {
    if state.unavailable() {
        MARK_DISABLED_BORDER
    } else if state.focus_border() {
        BORDER_FOCUS
    } else if state.active() {
        ACCENT
    } else {
        color_attribute(metadata, "border_color").unwrap_or(MARK_IDLE_BORDER)
    }
}

fn radio_background<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SelectionRenderState,
) -> &'a str {
    if state.unavailable() {
        MARK_DISABLED_FILL
    } else if state.active() {
        RADIO_CHECKED_FILL
    } else {
        color_attribute(metadata, "background_color").unwrap_or(MARK_IDLE_FILL)
    }
}

fn radio_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SelectionRenderState,
) -> &'a str {
    if state.unavailable() {
        MARK_DISABLED_BORDER
    } else if state.focus_border() {
        BORDER_FOCUS
    } else if state.active() {
        RADIO_CHECKED_BORDER
    } else {
        color_attribute(metadata, "border_color").unwrap_or(MARK_IDLE_BORDER)
    }
}

fn radio_dot_color<'a>(
    _metadata: &'a UiTemplateNodeMetadata,
    state: &SelectionRenderState,
) -> &'a str {
    if state.unavailable() {
        LABEL_DISABLED
    } else {
        ACCENT
    }
}

fn toggle_track_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SelectionRenderState,
) -> &'a str {
    if state.unavailable() {
        MARK_DISABLED_FILL
    } else if state.active() {
        SURFACE_SELECTED
    } else if state.pressed() {
        TOGGLE_PRESSED
    } else if state.surface_hot() {
        TOGGLE_HOVER
    } else {
        color_attribute(metadata, "background_color").unwrap_or(TOGGLE_TRACK_IDLE)
    }
}

fn toggle_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SelectionRenderState,
) -> &'a str {
    if state.unavailable() {
        MARK_DISABLED_BORDER
    } else if state.focus_border() {
        color_attribute(metadata, "border_color").unwrap_or(BORDER_FOCUS)
    } else if state.active() {
        TOGGLE_BORDER_ON
    } else {
        color_attribute(metadata, "border_color").unwrap_or(MARK_IDLE_BORDER)
    }
}

fn toggle_thumb_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SelectionRenderState,
) -> &'a str {
    if state.unavailable() {
        LABEL_DISABLED
    } else if state.active() {
        TOGGLE_THUMB_ON
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TOGGLE_THUMB_IDLE)
    }
}

fn label_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &SelectionRenderState) -> &'a str {
    if state.unavailable() {
        LABEL_DISABLED
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "foreground_color"))
            .unwrap_or(LABEL_MUTED)
    }
}

fn control_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "text", "value_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

fn label_gap(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "layout_spacing").unwrap_or(LABEL_GAP)
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
    state: &SelectionRenderState,
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
    state: &SelectionRenderState,
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
            font_size: LABEL_FONT_SIZE,
            line_height: LABEL_LINE_HEIGHT,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}
