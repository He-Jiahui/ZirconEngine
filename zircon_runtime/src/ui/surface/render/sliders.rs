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

const TRACK_HEIGHT: f32 = 3.0;
const TRACK_RADIUS: f32 = 2.0;
const THUMB_SIZE: f32 = 11.0;
const THUMB_HALO_SIZE: f32 = 20.0;
const HORIZONTAL_INSET: f32 = 8.0;
const LABEL_WIDTH: f32 = 50.0;
const LABEL_GAP: f32 = 12.0;
const VALUE_WIDTH: f32 = 44.0;
const VALUE_GAP: f32 = 10.0;
const FONT_SIZE: f32 = 11.0;
const LINE_HEIGHT: f32 = FONT_SIZE * 1.2;
const TRACK: &str = "#364046";
const TRACK_DISABLED: &str = "#262d32";
const VALUE_SURFACE: &str = "#11161a";
const VALUE_BORDER: &str = "#2d3940";
const BORDER_DISABLED: &str = "#343f47";
const TEXT: &str = "#aebdc4";
const TEXT_DISABLED: &str = "#59656c";
const THUMB: &str = "#c9f2f6";
const HALO: &str = "#35c7d03a";
const TICK: &str = "#50606a";
const ACCENT: &str = "#35c7d0";
const WARNING: &str = "#f5bd4f";
const ERROR: &str = "#ff735f";
const DISABLED_SURFACE: &str = "#252c31";

pub(super) fn slider_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_slider)
}

pub(super) fn slider_render_commands(
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
    if !is_slider(metadata) {
        return Vec::new();
    }
    let frame = pixel_aligned_frame(frame);
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = SliderRenderState::resolve(metadata, state_flags, component_state);
    let label = slider_label(metadata);
    let value_rect = slider_value_rect(frame);
    let track_rect = slider_track_rect(metadata, frame, value_rect, label.is_some());
    if track_rect.width <= 1.0 {
        return Vec::new();
    }

    let percent = slider_percent(metadata);
    let range_min = range_min_percent(metadata);
    let mut commands = Vec::new();
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            label_rect(frame),
            clip_frame,
            z_index.saturating_add(3),
            label,
            label_color(metadata, &state),
            &state,
            opacity,
        ));
    }
    push_track_commands(
        &mut commands,
        node_id,
        metadata,
        &state,
        track_rect,
        clip_frame,
        z_index,
        percent,
        range_min,
        opacity,
    );
    if let Some(ticks) = tick_count(metadata) {
        push_tick_commands(
            &mut commands,
            node_id,
            &state,
            track_rect,
            clip_frame,
            z_index.saturating_add(2),
            ticks,
            opacity,
        );
    }
    if let Some(range_min) = range_min {
        push_thumb_command(
            &mut commands,
            node_id,
            metadata,
            &state,
            track_rect,
            clip_frame,
            z_index.saturating_add(3),
            range_min,
            opacity,
        );
    }
    push_thumb_command(
        &mut commands,
        node_id,
        metadata,
        &state,
        track_rect,
        clip_frame,
        z_index.saturating_add(4),
        percent,
        opacity,
    );
    if let Some(range_min) = range_min {
        push_range_min_value(
            &mut commands,
            node_id,
            metadata,
            &state,
            frame,
            track_rect,
            clip_frame,
            z_index.saturating_add(5),
            range_min,
            opacity,
        );
    }
    if let Some(value_rect) = value_rect {
        push_value_box(
            &mut commands,
            node_id,
            metadata,
            &state,
            value_rect,
            clip_frame,
            z_index.saturating_add(5),
            percent,
            opacity,
        );
    }
    commands
}

#[derive(Clone, Copy)]
struct SliderRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl SliderRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        let family = UiPainterFamily::Slider;
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

    fn hot(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Pressed
                | UiPainterResolvedState::Focused
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}

fn is_slider(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "RangeField" | "Slider" | "RangeSlider"
    )
}

fn push_track_commands(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SliderRenderState,
    track: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    percent: f32,
    range_min: Option<f32>,
    opacity: f32,
) {
    commands.push(quad_command(
        node_id,
        track,
        clip_frame,
        z_index,
        track_color(metadata, state),
        None,
        0.0,
        TRACK_RADIUS,
        state,
        opacity,
    ));

    let (fill_start, fill_end) = fill_span(percent, range_min);
    let fill_width = (track.width * (fill_end - fill_start)).max(0.0);
    if fill_width > 0.0 {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                track.x + track.width * fill_start,
                track.y,
                fill_width.max(1.0),
                track.height,
            ),
            clip_frame,
            z_index.saturating_add(1),
            accent_color(metadata, state),
            None,
            0.0,
            TRACK_RADIUS,
            state,
            opacity,
        ));
    }
}

fn push_tick_commands(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    state: &SliderRenderState,
    track: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    tick_count: usize,
    opacity: f32,
) {
    if tick_count < 2 {
        return;
    }
    let last = tick_count - 1;
    for index in 0..tick_count {
        let fraction = index as f32 / last as f32;
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                track.x + track.width * fraction - 0.5,
                track.y + 8.0,
                1.0,
                4.0,
            ),
            clip_frame,
            z_index,
            tick_color(state),
            None,
            0.0,
            0.0,
            state,
            opacity,
        ));
    }
}

fn push_thumb_command(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SliderRenderState,
    track: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    percent: f32,
    opacity: f32,
) {
    let center_x = track.x + track.width * percent.clamp(0.0, 1.0);
    let center_y = track.y + track.height * 0.5;
    let thumb_size = number_attribute(metadata, "thumb_size")
        .or_else(|| number_attribute(metadata, "layout_icon_size"))
        .unwrap_or(THUMB_SIZE)
        .max(1.0);
    if state.hot() {
        commands.push(quad_command(
            node_id,
            centered_frame(center_x, center_y, THUMB_HALO_SIZE),
            clip_frame,
            z_index,
            color_attribute(metadata, "state_layer_color").unwrap_or(HALO),
            None,
            0.0,
            THUMB_HALO_SIZE * 0.5,
            state,
            opacity,
        ));
    }
    commands.push(quad_command(
        node_id,
        centered_frame(center_x, center_y, thumb_size),
        clip_frame,
        z_index.saturating_add(1),
        thumb_color(metadata, state),
        Some(thumb_outline_color(metadata, state)),
        1.0,
        thumb_size * 0.5,
        state,
        opacity,
    ));
}

fn push_value_box(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SliderRenderState,
    value_rect: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    percent: f32,
    opacity: f32,
) {
    commands.push(quad_command(
        node_id,
        value_rect,
        clip_frame,
        z_index,
        if state.unavailable() {
            DISABLED_SURFACE
        } else {
            VALUE_SURFACE
        },
        Some(value_border(metadata, state)),
        1.0,
        4.0,
        state,
        opacity,
    ));
    commands.push(text_command(
        node_id,
        UiFrame::new(
            value_rect.x + 6.0,
            value_rect.y + (value_rect.height - LINE_HEIGHT).max(0.0) * 0.5,
            (value_rect.width - 12.0).max(1.0),
            LINE_HEIGHT,
        ),
        clip_frame,
        z_index.saturating_add(1),
        value_label(metadata, percent),
        text_color(metadata, state),
        state,
        opacity,
    ));
}

fn push_range_min_value(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SliderRenderState,
    frame: UiFrame,
    track: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    range_min: f32,
    opacity: f32,
) {
    if frame.height < 42.0 || track.width < VALUE_WIDTH {
        return;
    }
    let value_rect = UiFrame::new(track.x, track.y + 10.0, VALUE_WIDTH, 20.0);
    commands.push(quad_command(
        node_id,
        value_rect,
        clip_frame,
        z_index,
        if state.unavailable() {
            DISABLED_SURFACE
        } else {
            VALUE_SURFACE
        },
        Some(range_value_border(state)),
        1.0,
        4.0,
        state,
        opacity,
    ));
    commands.push(text_command(
        node_id,
        UiFrame::new(
            value_rect.x + 6.0,
            value_rect.y + (value_rect.height - LINE_HEIGHT).max(0.0) * 0.5,
            (value_rect.width - 12.0).max(1.0),
            LINE_HEIGHT,
        ),
        clip_frame,
        z_index.saturating_add(1),
        format!("{:.2}", range_min.clamp(0.0, 1.0)),
        text_color(metadata, state),
        state,
        opacity,
    ));
}

fn slider_track_rect(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    value_rect: Option<UiFrame>,
    has_label: bool,
) -> UiFrame {
    let label_lane = if has_label {
        LABEL_WIDTH + LABEL_GAP
    } else {
        0.0
    };
    let left = frame.x
        + label_lane
        + HORIZONTAL_INSET
        + number_attribute(metadata, "layout_content_offset_x").unwrap_or(0.0);
    let right = (value_rect
        .map(|value| value.x - VALUE_GAP)
        .unwrap_or(frame.x + frame.width - HORIZONTAL_INSET)
        + number_attribute(metadata, "layout_first_cell_offset_x").unwrap_or(0.0))
    .max(left);
    UiFrame::new(
        left,
        frame.y + (frame.height - TRACK_HEIGHT).max(0.0) * 0.5,
        right - left,
        TRACK_HEIGHT,
    )
}

fn slider_value_rect(frame: UiFrame) -> Option<UiFrame> {
    if frame.width < 132.0 {
        return None;
    }
    let height = (frame.height - 6.0).clamp(18.0, 24.0);
    Some(UiFrame::new(
        frame.x + frame.width - HORIZONTAL_INSET - VALUE_WIDTH,
        frame.y + (frame.height - height).max(0.0) * 0.5,
        VALUE_WIDTH,
        height,
    ))
}

fn label_rect(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x + HORIZONTAL_INSET,
        frame.y + (frame.height - LINE_HEIGHT).max(0.0) * 0.5,
        LABEL_WIDTH,
        LINE_HEIGHT,
    )
}

fn slider_percent(metadata: &UiTemplateNodeMetadata) -> f32 {
    if let Some(percent) = number_attribute(metadata, "value_percent") {
        return declared_percent(percent);
    }
    let value = number_attribute(metadata, "value").unwrap_or(0.0);
    let min = number_attribute(metadata, "min").unwrap_or(0.0);
    let max = number_attribute(metadata, "max").unwrap_or(1.0);
    if (max - min).abs() <= f32::EPSILON {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

fn range_min_percent(metadata: &UiTemplateNodeMetadata) -> Option<f32> {
    number_attribute(metadata, "range_min_percent")
        .or_else(|| number_attribute(metadata, "layout_second_cell_offset_x"))
        .or_else(|| number_attribute(metadata, "range_min"))
        .map(declared_percent)
}

fn tick_count(metadata: &UiTemplateNodeMetadata) -> Option<usize> {
    number_attribute(metadata, "tick_count")
        .or_else(|| number_attribute(metadata, "steps"))
        .or_else(|| number_attribute(metadata, "layout_third_cell_offset_x"))
        .map(|value| value.round() as usize)
        .filter(|value| *value >= 2)
}

fn declared_percent(value: f32) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn fill_span(percent: f32, range_min: Option<f32>) -> (f32, f32) {
    let end = percent.clamp(0.0, 1.0);
    let start = range_min.unwrap_or(0.0).clamp(0.0, 1.0);
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

fn slider_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "label_text", "text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

fn value_label(metadata: &UiTemplateNodeMetadata, percent: f32) -> String {
    string_attribute(metadata, "value_text")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("{:.2}", percent.clamp(0.0, 1.0)))
}

fn track_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &SliderRenderState) -> &'a str {
    if state.unavailable() {
        TRACK_DISABLED
    } else {
        color_attribute(metadata, "track_color")
            .or_else(|| color_attribute(metadata, "background_color"))
            .unwrap_or(TRACK)
    }
}

fn accent_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &SliderRenderState) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if string_attribute(metadata, "validation_level").is_some_and(|level| level == "warning")
    {
        WARNING
    } else if string_attribute(metadata, "validation_level")
        .is_some_and(|level| matches!(level, "error" | "danger"))
    {
        ERROR
    } else {
        color_attribute(metadata, "value_color")
            .or_else(|| color_attribute(metadata, "accent_color"))
            .unwrap_or(ACCENT)
    }
}

fn thumb_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &SliderRenderState) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "thumb_color")
            .or_else(|| color_attribute(metadata, "icon_color"))
            .unwrap_or(THUMB)
    }
}

fn thumb_outline_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SliderRenderState,
) -> &'a str {
    if state.unavailable() {
        BORDER_DISABLED
    } else {
        color_attribute(metadata, "thumb_outline_color")
            .or_else(|| color_attribute(metadata, "border_color"))
            .unwrap_or_else(|| accent_color(metadata, state))
    }
}

fn label_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &SliderRenderState) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "label_color").unwrap_or(TEXT)
    }
}

fn text_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &SliderRenderState) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TEXT)
    }
}

fn value_border<'a>(metadata: &'a UiTemplateNodeMetadata, state: &SliderRenderState) -> &'a str {
    if state.unavailable() {
        BORDER_DISABLED
    } else if matches!(
        state.visual_state,
        UiPainterResolvedState::Focused | UiPainterResolvedState::Pressed
    ) {
        accent_color(metadata, state)
    } else {
        VALUE_BORDER
    }
}

fn range_value_border(state: &SliderRenderState) -> &'static str {
    if state.unavailable() {
        BORDER_DISABLED
    } else {
        VALUE_BORDER
    }
}

fn tick_color(state: &SliderRenderState) -> &'static str {
    if state.unavailable() {
        BORDER_DISABLED
    } else {
        TICK
    }
}

fn centered_frame(center_x: f32, center_y: f32, size: f32) -> UiFrame {
    UiFrame::new(center_x - size * 0.5, center_y - size * 0.5, size, size)
}

fn pixel_aligned_frame(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x.round(),
        frame.y.round(),
        frame.width.round().max(1.0),
        frame.height.round().max(1.0),
    )
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
    state: &SliderRenderState,
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
    state: &SliderRenderState,
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
            font_size: FONT_SIZE,
            line_height: LINE_HEIGHT,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}
