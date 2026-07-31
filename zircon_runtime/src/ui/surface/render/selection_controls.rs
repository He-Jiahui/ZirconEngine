use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

#[derive(Clone, Copy, Debug)]
struct SelectionVisual {
    label: UiRgbaColor,
    label_disabled: UiRgbaColor,
    mark_idle_fill: UiRgbaColor,
    mark_idle_border: UiRgbaColor,
    mark_disabled_fill: UiRgbaColor,
    mark_disabled_border: UiRgbaColor,
    selected_surface: UiRgbaColor,
    accent: UiRgbaColor,
    radio_checked_fill: UiRgbaColor,
    radio_checked_border: UiRgbaColor,
    toggle_idle: UiRgbaColor,
    toggle_thumb_idle: UiRgbaColor,
    toggle_thumb_active: UiRgbaColor,
    toggle_hover: UiRgbaColor,
    toggle_pressed: UiRgbaColor,
    mark_inset_x: f32,
    mark_size: f32,
    label_gap: f32,
    label_inset_y: f32,
    label_font_size: f32,
    label_line_height: f32,
    radio_dot_size: f32,
    toggle_track_width: f32,
    toggle_track_height: f32,
    toggle_thumb_size: f32,
    toggle_right_inset: f32,
    toggle_thumb_inset: f32,
    border_width: f32,
    mark_radius: f32,
    min_frame_extent: f32,
}

impl SelectionVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_selection_visual();
        visual.label = first_rgba_attribute(metadata, &["label_color", "foreground_color"])
            .unwrap_or(visual.label);
        visual.label_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.label_disabled);
        visual.mark_idle_fill =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.mark_idle_fill);
        visual.mark_idle_border =
            first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.mark_idle_border);
        visual.mark_disabled_fill = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.mark_disabled_fill);
        visual.mark_disabled_border = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.mark_disabled_border);
        visual.selected_surface = first_rgba_attribute(metadata, &["selected_background_color"])
            .unwrap_or(visual.selected_surface);
        visual.accent = first_rgba_attribute(metadata, &["accent_color", "focus_border_color"])
            .unwrap_or(visual.accent);
        visual.radio_checked_fill = first_rgba_attribute(metadata, &["checked_background_color"])
            .unwrap_or(visual.radio_checked_fill);
        visual.radio_checked_border = first_rgba_attribute(metadata, &["checked_border_color"])
            .unwrap_or(visual.radio_checked_border);
        visual.toggle_idle =
            first_rgba_attribute(metadata, &["toggle_background_color", "background_color"])
                .unwrap_or(visual.toggle_idle);
        visual.toggle_thumb_idle =
            first_rgba_attribute(metadata, &["thumb_color", "foreground_color"])
                .unwrap_or(visual.toggle_thumb_idle);
        visual.toggle_thumb_active = first_rgba_attribute(metadata, &["selected_thumb_color"])
            .unwrap_or(visual.toggle_thumb_active);
        visual.toggle_hover = first_rgba_attribute(metadata, &["hover_background_color"])
            .unwrap_or(visual.toggle_hover);
        visual.toggle_pressed = first_rgba_attribute(metadata, &["pressed_background_color"])
            .unwrap_or(visual.toggle_pressed);
        visual.mark_size = metric_attribute(metadata, "layout_icon_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.mark_size);
        visual.label_gap = metric_attribute(metadata, "layout_spacing")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.label_gap);
        visual.radio_dot_size = metric_attribute(metadata, "dot_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.radio_dot_size);
        visual.toggle_track_width = metric_attribute(metadata, "track_width")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.toggle_track_width);
        visual.toggle_track_height = metric_attribute(metadata, "track_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.toggle_track_height);
        visual.toggle_thumb_size = metric_attribute(metadata, "thumb_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.toggle_thumb_size);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.mark_radius = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.mark_radius);
        visual.label_font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.label_font_size);
        visual.label_line_height = line_height(
            metadata,
            "line_height",
            "line_height_ratio",
            visual.label_font_size,
            visual.label_line_height,
        );
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_selection_visual() -> &'static SelectionVisual {
    static VISUAL: OnceLock<SelectionVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        let border = controls.border_width;
        let track_height = controls.dense_height - density.gap_medium - border * 2.0;
        SelectionVisual {
            label: colors.text_secondary,
            label_disabled: colors.text_disabled,
            mark_idle_fill: colors.surface_recessed,
            mark_idle_border: colors.separator_strong,
            mark_disabled_fill: colors.surface_disabled,
            mark_disabled_border: colors.border_disabled,
            selected_surface: colors.surface_selected,
            accent: colors.accent,
            radio_checked_fill: colors.surface[2],
            radio_checked_border: colors.border,
            toggle_idle: colors.surface[2],
            toggle_thumb_idle: colors.text_secondary,
            toggle_thumb_active: colors.text_primary,
            toggle_hover: colors.surface_hover,
            toggle_pressed: colors.surface[3],
            mark_inset_x: density.gap_medium + border * 2.0,
            mark_size: controls.dense_height - density.gap_large,
            label_gap: density.gap_medium + border,
            label_inset_y: density.gap_small + border,
            label_font_size: typography.body_size,
            label_line_height: typography.body_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            radio_dot_size: density.gap_small + border * 3.0,
            toggle_track_width: controls.default_height + border * 2.0,
            toggle_track_height: track_height,
            toggle_thumb_size: track_height - density.gap_small - border * 2.0,
            toggle_right_inset: density.gap_medium,
            toggle_thumb_inset: border * 2.0,
            border_width: border,
            mark_radius: controls.small_radius,
            min_frame_extent: border.max(f32::EPSILON),
        }
    })
}

pub(super) fn selection_control_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.and_then(selection_control_kind).is_some()
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
    let visual = SelectionVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let state = SelectionRenderState::resolve(metadata, state_flags, component_state);
    match kind {
        SelectionControlKind::Checkbox => checkbox_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
        SelectionControlKind::Radio => radio_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
        SelectionControlKind::Toggle => toggle_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
    }
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
        Self {
            family,
            checked,
            selected,
            visual_state: painter_state.resolved_state_for_family(family),
            surface_hot: painter_state.hovered
                || painter_state.dragging
                || painter_state.drop_hovered,
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

#[allow(clippy::too_many_arguments)]
fn checkbox_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mark = leading_mark_rect(frame, visual);
    let mut commands = vec![quad_command(
        node_id,
        mark,
        clip,
        z.saturating_add(1),
        checkbox_background(state, visual),
        Some(checkbox_border(state, visual)),
        visual.border_width,
        visual.mark_radius,
        state,
        opacity,
    )];
    if state.active() {
        commands.extend(checkbox_tick_commands(
            node_id,
            mark,
            clip,
            z.saturating_add(2),
            state,
            visual,
            opacity,
        ));
    }
    push_label(
        &mut commands,
        node_id,
        metadata,
        label_rect_after_mark(frame, mark, visual),
        clip,
        z.saturating_add(4),
        state,
        visual,
        opacity,
    );
    commands
}

#[allow(clippy::too_many_arguments)]
fn radio_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mark = leading_mark_rect(frame, visual);
    let mut commands = vec![quad_command(
        node_id,
        mark,
        clip,
        z.saturating_add(1),
        radio_background(state, visual),
        Some(radio_border(state, visual)),
        visual.border_width,
        mark.height * 0.5,
        state,
        opacity,
    )];
    if state.active() {
        commands.push(quad_command(
            node_id,
            centered_square(mark, visual.radio_dot_size),
            clip,
            z.saturating_add(2),
            radio_dot(state, visual),
            None,
            0.0,
            visual.radio_dot_size * 0.5,
            state,
            opacity,
        ));
    }
    push_label(
        &mut commands,
        node_id,
        metadata,
        label_rect_after_mark(frame, mark, visual),
        clip,
        z.saturating_add(4),
        state,
        visual,
        opacity,
    );
    commands
}

#[allow(clippy::too_many_arguments)]
fn toggle_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let track = toggle_track_rect(frame, visual);
    let mut commands = Vec::new();
    push_label(
        &mut commands,
        node_id,
        metadata,
        toggle_label_rect(frame, track, visual),
        clip,
        z.saturating_add(3),
        state,
        visual,
        opacity,
    );
    commands.push(quad_command(
        node_id,
        track,
        clip,
        z.saturating_add(1),
        toggle_track(state, visual),
        Some(toggle_border(state, visual)),
        visual.border_width,
        track.height * 0.5,
        state,
        opacity,
    ));
    let thumb = toggle_thumb_rect(state, track, visual);
    commands.push(quad_command(
        node_id,
        thumb,
        clip,
        z.saturating_add(2),
        toggle_thumb(state, visual),
        None,
        0.0,
        thumb.height * 0.5,
        state,
        opacity,
    ));
    commands
}

#[allow(clippy::too_many_arguments)]
fn push_label(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    opacity: f32,
) {
    let Some(label) = control_label(metadata) else {
        return;
    };
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return;
    }
    commands.push(text_command(
        node_id,
        frame,
        clip,
        z,
        label,
        label_color(state, visual),
        visual.label_font_size,
        visual.label_line_height,
        state,
        opacity,
    ));
}

fn checkbox_tick_commands(
    node_id: UiNodeId,
    mark: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let unit = mark.width * (3.0 / 16.0);
    [
        (3.0, 7.0, 3.0, 3.0),
        (5.0, 9.0, 3.0, 3.0),
        (8.0, 4.0, 3.0, 8.0),
    ]
    .into_iter()
    .map(|(x, y, w, h)| {
        quad_command(
            node_id,
            UiFrame::new(
                mark.x + x * unit / 3.0,
                mark.y + y * unit / 3.0,
                w * unit / 3.0,
                h * unit / 3.0,
            ),
            clip,
            z,
            visual.accent,
            None,
            0.0,
            visual.border_width,
            state,
            opacity,
        )
    })
    .collect()
}

fn leading_mark_rect(frame: UiFrame, visual: &SelectionVisual) -> UiFrame {
    UiFrame::new(
        frame.x + visual.mark_inset_x,
        frame.y + (frame.height - visual.mark_size).max(0.0) * 0.5,
        visual.mark_size,
        visual.mark_size,
    )
}
fn label_rect_after_mark(frame: UiFrame, mark: UiFrame, visual: &SelectionVisual) -> UiFrame {
    let x = mark.x + mark.width + visual.label_gap;
    UiFrame::new(
        x,
        frame.y + visual.label_inset_y,
        (frame.x + frame.width - x - visual.mark_inset_x).max(visual.min_frame_extent),
        (frame.height - visual.label_inset_y * 2.0).max(visual.label_line_height),
    )
}
fn toggle_label_rect(frame: UiFrame, track: UiFrame, visual: &SelectionVisual) -> UiFrame {
    UiFrame::new(
        frame.x + visual.mark_inset_x,
        frame.y + visual.label_inset_y,
        (track.x - frame.x - visual.mark_inset_x - visual.label_gap).max(visual.min_frame_extent),
        (frame.height - visual.label_inset_y * 2.0).max(visual.label_line_height),
    )
}
fn toggle_track_rect(frame: UiFrame, visual: &SelectionVisual) -> UiFrame {
    let width = visual
        .toggle_track_width
        .min((frame.width - visual.mark_inset_x * 2.0).max(visual.min_frame_extent));
    let height = visual
        .toggle_track_height
        .min(frame.height.max(visual.min_frame_extent));
    UiFrame::new(
        (frame.x + frame.width - visual.toggle_right_inset - width).max(frame.x),
        frame.y + (frame.height - height).max(0.0) * 0.5,
        width,
        height,
    )
}
fn toggle_thumb_rect(
    state: &SelectionRenderState,
    track: UiFrame,
    visual: &SelectionVisual,
) -> UiFrame {
    let size = visual
        .toggle_thumb_size
        .min(track.width)
        .min(track.height)
        .max(visual.min_frame_extent);
    let available = (track.width - size - visual.toggle_thumb_inset * 2.0).max(0.0);
    UiFrame::new(
        track.x + visual.toggle_thumb_inset + if state.active() { available } else { 0.0 },
        track.y + (track.height - size).max(0.0) * 0.5,
        size,
        size,
    )
}
fn centered_square(frame: UiFrame, size: f32) -> UiFrame {
    let size = size.min(frame.width).min(frame.height).max(f32::EPSILON);
    UiFrame::new(
        frame.x + (frame.width - size).max(0.0) * 0.5,
        frame.y + (frame.height - size).max(0.0) * 0.5,
        size,
        size,
    )
}

fn checkbox_background(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_fill
    } else if state.active() {
        visual.selected_surface
    } else {
        visual.mark_idle_fill
    }
}
fn checkbox_border(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_border
    } else if state.focus_border() || state.active() {
        visual.accent
    } else {
        visual.mark_idle_border
    }
}
fn radio_background(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_fill
    } else if state.active() {
        visual.radio_checked_fill
    } else {
        visual.mark_idle_fill
    }
}
fn radio_border(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_border
    } else if state.focus_border() {
        visual.accent
    } else if state.active() {
        visual.radio_checked_border
    } else {
        visual.mark_idle_border
    }
}
fn radio_dot(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.label_disabled
    } else {
        visual.accent
    }
}
fn toggle_track(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_fill
    } else if state.active() {
        visual.selected_surface
    } else if state.pressed() {
        visual.toggle_pressed
    } else if state.surface_hot() {
        visual.toggle_hover
    } else {
        visual.toggle_idle
    }
}
fn toggle_border(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_border
    } else if state.focus_border() || state.active() {
        visual.accent
    } else {
        visual.mark_idle_border
    }
}
fn toggle_thumb(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.label_disabled
    } else if state.active() {
        visual.toggle_thumb_active
    } else {
        visual.toggle_thumb_idle
    }
}
fn label_color(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.label_disabled
    } else {
        visual.label
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
fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}
fn metric_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(value_as_f32)
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
fn value_as_f32(value: &Value) -> Option<f32> {
    let value = match value {
        Value::Integer(value) => *value as f64,
        Value::Float(value) if value.is_finite() => *value,
        _ => return None,
    } as f32;
    value.is_finite().then_some(value)
}
fn line_height(
    metadata: &UiTemplateNodeMetadata,
    absolute_key: &str,
    ratio_key: &str,
    font_size: f32,
    default: f32,
) -> f32 {
    metric_attribute(metadata, absolute_key)
        .filter(|value| *value > 0.0)
        .or_else(|| {
            metric_attribute(metadata, ratio_key)
                .filter(|value| *value > 0.0)
                .map(|ratio| font_size * ratio)
        })
        .unwrap_or(default)
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

#[allow(clippy::too_many_arguments)]
fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: UiRgbaColor,
    border: Option<UiRgbaColor>,
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
            background_color: Some(css_color(background)),
            border_color: border.map(css_color),
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
#[allow(clippy::too_many_arguments)]
fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
    font_size: f32,
    line_height: f32,
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
            foreground_color: Some(css_color(foreground)),
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
