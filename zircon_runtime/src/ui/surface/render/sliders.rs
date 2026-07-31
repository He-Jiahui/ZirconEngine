use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
    surface::{bounded_ui_slider_tick_count, ui_slider_tick_count_for_track, UiRenderCommand},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

mod commands;
mod state_colors;

use commands::{quad_command, text_command};
use state_colors::{
    fill_color, halo_color, label_color, range_value_border, text_color, thumb_color,
    thumb_outline_color, tick_color, track_color, value_border, value_surface_color,
};

#[derive(Clone, Copy, Debug)]
struct SliderVisual {
    track: UiRgbaColor,
    track_disabled: UiRgbaColor,
    value_surface: UiRgbaColor,
    value_surface_disabled: UiRgbaColor,
    value_border: UiRgbaColor,
    border_disabled: UiRgbaColor,
    label_text: UiRgbaColor,
    text: UiRgbaColor,
    text_disabled: UiRgbaColor,
    thumb: UiRgbaColor,
    thumb_outline: UiRgbaColor,
    halo: UiRgbaColor,
    tick: UiRgbaColor,
    fill: UiRgbaColor,
    warning: UiRgbaColor,
    error: UiRgbaColor,
    track_height: f32,
    track_radius: f32,
    thumb_size: f32,
    thumb_halo_size: f32,
    horizontal_inset: f32,
    label_width: f32,
    label_gap: f32,
    value_width: f32,
    value_gap: f32,
    value_text_inset: f32,
    value_corner_radius: f32,
    value_min_height: f32,
    range_value_min_frame_height: f32,
    range_value_top: f32,
    tick_width: f32,
    tick_height: f32,
    tick_offset_y: f32,
    font_size: f32,
    line_height: f32,
    border_width: f32,
    min_frame_extent: f32,
}

impl SliderVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_slider_visual();
        visual.track = first_rgba_attribute(metadata, &["track_color", "background_color"])
            .unwrap_or(visual.track);
        visual.track_disabled = first_rgba_attribute(metadata, &["disabled_track_color"])
            .unwrap_or(visual.track_disabled);
        visual.value_surface = first_rgba_attribute(metadata, &["value_background_color"])
            .unwrap_or(visual.value_surface);
        visual.value_surface_disabled =
            first_rgba_attribute(metadata, &["disabled_background_color"])
                .unwrap_or(visual.value_surface_disabled);
        visual.value_border =
            first_rgba_attribute(metadata, &["value_border_color", "border_color"])
                .unwrap_or(visual.value_border);
        visual.border_disabled = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.border_disabled);
        visual.label_text =
            first_rgba_attribute(metadata, &["label_color"]).unwrap_or(visual.label_text);
        visual.text = first_rgba_attribute(metadata, &["foreground_color"]).unwrap_or(visual.text);
        visual.text_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.text_disabled);
        visual.thumb =
            first_rgba_attribute(metadata, &["thumb_color", "icon_color"]).unwrap_or(visual.thumb);
        visual.thumb_outline =
            first_rgba_attribute(metadata, &["thumb_outline_color", "border_color"])
                .unwrap_or(visual.thumb_outline);
        visual.halo = first_rgba_attribute(metadata, &["state_layer_color"]).unwrap_or(visual.halo);
        visual.tick = first_rgba_attribute(metadata, &["tick_color"]).unwrap_or(visual.tick);
        visual.fill =
            first_rgba_attribute(metadata, &["value_color", "accent_color"]).unwrap_or(visual.fill);
        visual.warning =
            first_rgba_attribute(metadata, &["warning_color"]).unwrap_or(visual.warning);
        visual.error = first_rgba_attribute(metadata, &["error_color"]).unwrap_or(visual.error);
        visual.track_height = metric_attribute(metadata, "track_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.track_height);
        visual.thumb_size = metric_attribute(metadata, "thumb_size")
            .or_else(|| metric_attribute(metadata, "layout_icon_size"))
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.thumb_size);
        visual.thumb_halo_size = metric_attribute(metadata, "thumb_halo_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.thumb_halo_size);
        visual.horizontal_inset = metric_attribute(metadata, "horizontal_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.horizontal_inset);
        visual.label_width = metric_attribute(metadata, "label_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.label_width);
        visual.label_gap = metric_attribute(metadata, "label_gap")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.label_gap);
        visual.value_width = metric_attribute(metadata, "value_width")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.value_width);
        visual.value_gap = metric_attribute(metadata, "value_gap")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.value_gap);
        visual.value_text_inset = metric_attribute(metadata, "value_text_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.value_text_inset);
        visual.value_corner_radius = metric_attribute(metadata, "value_corner_radius")
            .or_else(|| metric_attribute(metadata, "corner_radius"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.value_corner_radius);
        visual.value_min_height = metric_attribute(metadata, "value_min_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.value_min_height);
        visual.range_value_min_frame_height =
            metric_attribute(metadata, "range_value_min_frame_height")
                .filter(|value| *value > 0.0)
                .unwrap_or(visual.range_value_min_frame_height);
        visual.range_value_top = metric_attribute(metadata, "range_value_top")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.range_value_top);
        visual.tick_width = metric_attribute(metadata, "tick_width")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.tick_width);
        visual.tick_height = metric_attribute(metadata, "tick_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.tick_height);
        visual.tick_offset_y = metric_attribute(metadata, "tick_offset_y")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.tick_offset_y);
        visual.font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.font_size);
        visual.line_height = line_height(
            metadata,
            "line_height",
            "line_height_ratio",
            visual.font_size,
            visual.line_height,
        );
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.track_radius = visual.track_height * 0.5;
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_slider_visual() -> &'static SliderVisual {
    static VISUAL: OnceLock<SliderVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        SliderVisual {
            track: colors.surface[0],
            track_disabled: colors.surface_disabled,
            value_surface: colors.surface[0],
            value_surface_disabled: colors.surface_disabled,
            value_border: colors.border,
            border_disabled: colors.border_disabled,
            label_text: colors.text_secondary,
            text: colors.text_primary,
            text_disabled: colors.text_disabled,
            thumb: colors.text_primary,
            thumb_outline: colors.border,
            halo: with_alpha(
                colors.text_primary,
                controls.border_width / density.gap_medium,
            ),
            tick: colors.separator_soft,
            fill: colors.separator_strong,
            warning: colors.warning,
            error: colors.error,
            track_height: density.gap_small,
            track_radius: density.gap_small * 0.5,
            thumb_size: density.gap_medium,
            thumb_halo_size: density.gap_medium * 2.0,
            horizontal_inset: density.gap_medium,
            label_width: controls.default_height + density.gap_large + density.gap_medium
                - controls.border_width * 2.0,
            label_gap: density.gap_large,
            value_width: controls.dense_height + density.gap_large + controls.border_width * 4.0,
            value_gap: density.gap_large - controls.border_width * 2.0,
            value_text_inset: density.gap_medium - controls.border_width * 2.0,
            value_corner_radius: controls.small_radius,
            value_min_height: controls.compact_height
                - density.gap_medium
                - controls.border_width * 2.0,
            range_value_min_frame_height: controls.default_height
                + density.gap_medium
                + controls.border_width * 2.0,
            range_value_top: density.gap_large - controls.border_width * 2.0,
            tick_width: controls.border_width,
            tick_height: density.gap_small,
            tick_offset_y: density.gap_medium,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            border_width: controls.border_width,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

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
    let visual = SliderVisual::resolve(metadata);
    let frame = pixel_aligned_frame(frame);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }

    let state = SliderRenderState::resolve(metadata, state_flags, component_state);
    let label = slider_label(metadata);
    let value_rect = slider_value_rect(frame, &visual);
    let track_rect = slider_track_rect(metadata, frame, value_rect, label.is_some(), &visual);
    if track_rect.width <= visual.min_frame_extent {
        return Vec::new();
    }

    let percent = slider_percent(metadata);
    let range_min = range_min_percent(metadata);
    let mut commands = Vec::new();
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            label_rect(frame, &visual),
            clip_frame,
            z_index.saturating_add(3),
            label,
            label_color(&state, &visual),
            &visual,
            &state,
            opacity,
        ));
    }
    push_track_commands(
        &mut commands,
        node_id,
        metadata,
        &state,
        &visual,
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
            &visual,
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
            &state,
            &visual,
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
        &state,
        &visual,
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
            &visual,
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
            &visual,
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
    surface_hot: bool,
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
            surface_hot: painter_state.hovered
                || painter_state.dragging
                || painter_state.drop_hovered,
        }
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

    fn thumb_halo(self) -> bool {
        self.pressed() || self.focused() || self.surface_hot
    }
}

fn is_slider(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "RangeField" | "Slider" | "RangeSlider"
    )
}

#[allow(clippy::too_many_arguments)]
fn push_track_commands(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SliderRenderState,
    visual: &SliderVisual,
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
        track_color(state, visual),
        None,
        0.0,
        visual.track_radius,
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
                fill_width.max(visual.min_frame_extent),
                track.height,
            ),
            clip_frame,
            z_index.saturating_add(1),
            fill_color(metadata, state, visual),
            None,
            0.0,
            visual.track_radius,
            state,
            opacity,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn push_tick_commands(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    state: &SliderRenderState,
    visual: &SliderVisual,
    track: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    tick_count: usize,
    opacity: f32,
) {
    let tick_count = ui_slider_tick_count_for_track(tick_count, track.width);
    if tick_count < 2 {
        return;
    }
    let last = tick_count - 1;
    for index in 0..tick_count {
        let fraction = index as f32 / last as f32;
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                track.x + track.width * fraction - visual.tick_width * 0.5,
                track.y + visual.tick_offset_y,
                visual.tick_width,
                visual.tick_height,
            ),
            clip_frame,
            z_index,
            tick_color(state, visual),
            None,
            0.0,
            0.0,
            state,
            opacity,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn push_thumb_command(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    state: &SliderRenderState,
    visual: &SliderVisual,
    track: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    percent: f32,
    opacity: f32,
) {
    let center_x = track.x + track.width * percent.clamp(0.0, 1.0);
    let center_y = track.y + track.height * 0.5;
    if state.thumb_halo() {
        commands.push(quad_command(
            node_id,
            centered_frame(center_x, center_y, visual.thumb_halo_size),
            clip_frame,
            z_index,
            halo_color(state, visual),
            None,
            0.0,
            visual.thumb_halo_size * 0.5,
            state,
            opacity,
        ));
    }
    commands.push(quad_command(
        node_id,
        centered_frame(center_x, center_y, visual.thumb_size),
        clip_frame,
        z_index.saturating_add(1),
        thumb_color(state, visual),
        Some(thumb_outline_color(state, visual)),
        visual.border_width,
        visual.thumb_size * 0.5,
        state,
        opacity,
    ));
}

#[allow(clippy::too_many_arguments)]
fn push_value_box(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SliderRenderState,
    visual: &SliderVisual,
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
        value_surface_color(state, visual),
        Some(value_border(state, visual, metadata)),
        visual.border_width,
        visual.value_corner_radius,
        state,
        opacity,
    ));
    commands.push(text_command(
        node_id,
        UiFrame::new(
            value_rect.x + visual.value_text_inset,
            value_rect.y + (value_rect.height - visual.line_height).max(0.0) * 0.5,
            (value_rect.width - visual.value_text_inset * 2.0).max(visual.min_frame_extent),
            visual.line_height,
        ),
        clip_frame,
        z_index.saturating_add(1),
        value_label(metadata, percent),
        text_color(state, visual),
        visual,
        state,
        opacity,
    ));
}

#[allow(clippy::too_many_arguments)]
fn push_range_min_value(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SliderRenderState,
    visual: &SliderVisual,
    frame: UiFrame,
    track: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    range_min: f32,
    opacity: f32,
) {
    if frame.height < visual.range_value_min_frame_height || track.width < visual.value_width {
        return;
    }
    let value_rect = UiFrame::new(
        track.x,
        track.y + visual.range_value_top,
        visual.value_width,
        visual.value_min_height,
    );
    commands.push(quad_command(
        node_id,
        value_rect,
        clip_frame,
        z_index,
        value_surface_color(state, visual),
        Some(range_value_border(state, visual)),
        visual.border_width,
        visual.value_corner_radius,
        state,
        opacity,
    ));
    commands.push(text_command(
        node_id,
        UiFrame::new(
            value_rect.x + visual.value_text_inset,
            value_rect.y + (value_rect.height - visual.line_height).max(0.0) * 0.5,
            (value_rect.width - visual.value_text_inset * 2.0).max(visual.min_frame_extent),
            visual.line_height,
        ),
        clip_frame,
        z_index.saturating_add(1),
        format!("{:.2}", range_min.clamp(0.0, 1.0)),
        text_color(state, visual),
        visual,
        state,
        opacity,
    ));
}

fn slider_track_rect(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    value_rect: Option<UiFrame>,
    has_label: bool,
    visual: &SliderVisual,
) -> UiFrame {
    let label_lane = if has_label {
        visual.label_width + visual.label_gap
    } else {
        0.0
    };
    let left = frame.x
        + label_lane
        + visual.horizontal_inset
        + data_number_attribute(metadata, "layout_content_offset_x").unwrap_or(0.0);
    let right = (value_rect
        .map(|value| value.x - visual.value_gap)
        .unwrap_or(frame.x + frame.width - visual.horizontal_inset)
        + data_number_attribute(metadata, "layout_first_cell_offset_x").unwrap_or(0.0))
    .max(left);
    UiFrame::new(
        left,
        frame.y + (frame.height - visual.track_height).max(0.0) * 0.5,
        right - left,
        visual.track_height,
    )
}

fn slider_value_rect(frame: UiFrame, visual: &SliderVisual) -> Option<UiFrame> {
    if frame.width < visual.value_width + visual.horizontal_inset * 2.0 + visual.value_gap {
        return None;
    }
    let height = (frame.height - visual.value_text_inset).clamp(
        visual.value_min_height,
        visual.value_min_height + visual.horizontal_inset - visual.border_width * 2.0,
    );
    Some(UiFrame::new(
        frame.x + frame.width - visual.horizontal_inset - visual.value_width,
        frame.y + (frame.height - height).max(0.0) * 0.5,
        visual.value_width,
        height,
    ))
}

fn label_rect(frame: UiFrame, visual: &SliderVisual) -> UiFrame {
    UiFrame::new(
        frame.x + visual.horizontal_inset,
        frame.y + (frame.height - visual.line_height).max(0.0) * 0.5,
        visual.label_width,
        visual.line_height,
    )
}

fn slider_percent(metadata: &UiTemplateNodeMetadata) -> f32 {
    if let Some(percent) = data_number_attribute(metadata, "value_percent") {
        return declared_percent(percent);
    }
    let value = data_number_attribute(metadata, "value").unwrap_or(0.0);
    let min = data_number_attribute(metadata, "min").unwrap_or(0.0);
    let max = data_number_attribute(metadata, "max").unwrap_or(1.0);
    if (max - min).abs() <= f32::EPSILON {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

fn range_min_percent(metadata: &UiTemplateNodeMetadata) -> Option<f32> {
    data_number_attribute(metadata, "range_min_percent")
        .or_else(|| data_number_attribute(metadata, "layout_second_cell_offset_x"))
        .or_else(|| data_number_attribute(metadata, "range_min"))
        .map(declared_percent)
}

fn tick_count(metadata: &UiTemplateNodeMetadata) -> Option<usize> {
    data_number_attribute(metadata, "tick_count")
        .or_else(|| data_number_attribute(metadata, "steps"))
        .or_else(|| data_number_attribute(metadata, "layout_third_cell_offset_x"))
        .and_then(bounded_ui_slider_tick_count)
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

fn data_number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(value_as_f32)
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

fn with_alpha(color: UiRgbaColor, alpha: f32) -> UiRgbaColor {
    UiRgbaColor::new(color.red, color.green, color.blue, alpha)
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
