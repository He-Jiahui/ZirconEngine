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
struct ProgressVisual {
    track: UiRgbaColor,
    track_disabled: UiRgbaColor,
    fill: UiRgbaColor,
    fill_disabled: UiRgbaColor,
    border: UiRgbaColor,
    border_disabled: UiRgbaColor,
    label: UiRgbaColor,
    label_disabled: UiRgbaColor,
    warning: UiRgbaColor,
    error: UiRgbaColor,
    track_height: f32,
    horizontal_inset: f32,
    corner_radius: f32,
    border_width: f32,
    label_gap: f32,
    font_size: f32,
    line_height: f32,
    min_frame_extent: f32,
}

impl ProgressVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_progress_visual();
        visual.track = first_rgba_attribute(metadata, &["track_color", "background_color"])
            .unwrap_or(visual.track);
        visual.track_disabled = first_rgba_attribute(metadata, &["disabled_track_color"])
            .unwrap_or(visual.track_disabled);
        visual.fill =
            first_rgba_attribute(metadata, &["fill_color", "value_color", "accent_color"])
                .unwrap_or(visual.fill);
        visual.fill_disabled = first_rgba_attribute(metadata, &["disabled_fill_color"])
            .unwrap_or(visual.fill_disabled);
        visual.border = first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.border);
        visual.border_disabled = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.border_disabled);
        visual.label = first_rgba_attribute(metadata, &["label_color", "foreground_color"])
            .unwrap_or(visual.label);
        visual.label_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.label_disabled);
        visual.warning =
            first_rgba_attribute(metadata, &["warning_color"]).unwrap_or(visual.warning);
        visual.error = first_rgba_attribute(metadata, &["error_color"]).unwrap_or(visual.error);
        visual.track_height = metric_attribute(metadata, "track_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.track_height);
        visual.horizontal_inset = metric_attribute(metadata, "horizontal_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.horizontal_inset);
        visual.corner_radius = metric_attribute(metadata, "corner_radius")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.corner_radius);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.label_gap = metric_attribute(metadata, "label_gap")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.label_gap);
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
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_progress_visual() -> &'static ProgressVisual {
    static VISUAL: OnceLock<ProgressVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        ProgressVisual {
            track: colors.surface_recessed,
            track_disabled: colors.surface_disabled,
            fill: colors.accent,
            fill_disabled: colors.text_disabled,
            border: colors.separator_soft,
            border_disabled: colors.border_disabled,
            label: colors.text_primary,
            label_disabled: colors.text_disabled,
            warning: colors.warning,
            error: colors.error,
            track_height: density.gap_small,
            horizontal_inset: density.gap_medium,
            corner_radius: controls.small_radius,
            border_width: controls.border_width,
            label_gap: density.gap_small,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn progress_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_linear_progress)
}

pub(super) fn progress_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_linear_progress)
}

pub(super) fn progress_suppresses_owner_surface(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_linear_progress)
}

pub(super) fn progress_render_commands(
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
    if !is_linear_progress(metadata) {
        return Vec::new();
    }
    let visual = ProgressVisual::resolve(metadata);
    let frame = pixel_aligned_frame(frame);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }

    let state = ProgressRenderState::resolve(metadata, state_flags, component_state);
    let label = progress_label(metadata);
    let (track, label_frame) = progress_frames(frame, label.as_deref(), &visual);
    if track.width <= visual.min_frame_extent || track.height <= visual.min_frame_extent {
        return Vec::new();
    }

    let radius = visual.corner_radius.min(track.height * 0.5);
    let mut commands = vec![quad_command(
        node_id,
        track,
        clip_frame,
        z_index.saturating_add(1),
        track_color(&state, &visual),
        Some(border_color(&state, &visual)),
        visual.border_width,
        radius,
        &state,
        opacity,
    )];
    let percent = progress_percent(metadata);
    let fill_width = (track.width * percent).clamp(0.0, track.width);
    if fill_width > 0.0 {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                track.x,
                track.y,
                fill_width.max(visual.min_frame_extent).min(track.width),
                track.height,
            ),
            clip_frame,
            z_index.saturating_add(2),
            fill_color(metadata, &state, &visual),
            None,
            0.0,
            radius,
            &state,
            opacity,
        ));
    }
    if let (Some(label), Some(label_frame)) = (label, label_frame) {
        commands.push(text_command(
            node_id,
            label_frame,
            clip_frame,
            z_index.saturating_add(3),
            label,
            label_color(&state, &visual),
            &visual,
            &state,
            opacity,
        ));
    }
    commands
}

#[derive(Clone, Copy)]
struct ProgressRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl ProgressRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        let family = UiPainterFamily::Generic;
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
        }
    }

    fn disabled(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Disabled)
    }

    fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }
}

fn is_progress(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "Progress" | "ProgressBar" | "LinearProgress"
    )
}

fn is_linear_progress(metadata: &UiTemplateNodeMetadata) -> bool {
    is_progress(metadata)
        && !string_attribute(metadata, "variant")
            .is_some_and(|variant| variant.eq_ignore_ascii_case("circular"))
}

fn progress_frames(
    frame: UiFrame,
    label: Option<&str>,
    visual: &ProgressVisual,
) -> (UiFrame, Option<UiFrame>) {
    let inset = visual.horizontal_inset.min(frame.width * 0.5);
    let content = UiFrame::new(
        frame.x + inset,
        frame.y,
        (frame.width - inset * 2.0).max(0.0),
        frame.height,
    );
    let total_label_height = visual.line_height + visual.label_gap + visual.track_height;
    if label.is_some() && content.height >= total_label_height {
        let top = content.y + (content.height - total_label_height) * 0.5;
        return (
            UiFrame::new(
                content.x,
                top + visual.line_height + visual.label_gap,
                content.width,
                visual.track_height,
            ),
            Some(UiFrame::new(
                content.x,
                top,
                content.width,
                visual.line_height,
            )),
        );
    }
    (
        UiFrame::new(
            content.x,
            content.y + (content.height - visual.track_height).max(0.0) * 0.5,
            content.width,
            visual.track_height.min(content.height),
        ),
        None,
    )
}

fn progress_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    if !bool_attribute(metadata, "show_label").unwrap_or(false) {
        return None;
    }
    string_attribute(metadata, "label_text")
        .or_else(|| string_attribute(metadata, "label"))
        .filter(|text| !text.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn progress_percent(metadata: &UiTemplateNodeMetadata) -> f32 {
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

fn declared_percent(value: f32) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn track_color(state: &ProgressRenderState, visual: &ProgressVisual) -> UiRgbaColor {
    if state.disabled() {
        visual.track_disabled
    } else {
        visual.track
    }
}

fn border_color(state: &ProgressRenderState, visual: &ProgressVisual) -> UiRgbaColor {
    if state.disabled() {
        visual.border_disabled
    } else if state.focused() {
        visual.fill
    } else {
        visual.border
    }
}

fn fill_color(
    metadata: &UiTemplateNodeMetadata,
    state: &ProgressRenderState,
    visual: &ProgressVisual,
) -> UiRgbaColor {
    if state.disabled() {
        visual.fill_disabled
    } else if severity(metadata).is_some_and(|severity| severity == "warning") {
        visual.warning
    } else if severity(metadata).is_some_and(|severity| matches!(severity, "error" | "danger")) {
        visual.error
    } else {
        visual.fill
    }
}

fn label_color(state: &ProgressRenderState, visual: &ProgressVisual) -> UiRgbaColor {
    if state.disabled() {
        visual.label_disabled
    } else {
        visual.label
    }
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
    state: &ProgressRenderState,
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
    visual: &ProgressVisual,
    state: &ProgressRenderState,
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
            font_size: visual.font_size,
            line_height: visual.line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
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

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn severity(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    string_attribute(metadata, "validation_level").or_else(|| string_attribute(metadata, "status"))
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

fn pixel_aligned_frame(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x.round(),
        frame.y.round(),
        frame.width.round().max(1.0),
        frame.height.round().max(1.0),
    )
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
