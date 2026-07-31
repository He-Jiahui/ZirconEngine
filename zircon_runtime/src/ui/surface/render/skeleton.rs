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

#[derive(Clone, Copy, Debug)]
struct SkeletonVisual {
    background: UiRgbaColor,
    background_disabled: UiRgbaColor,
    highlight: UiRgbaColor,
    highlight_disabled: UiRgbaColor,
    corner_radius: f32,
    border_width: f32,
    min_frame_extent: f32,
}

impl SkeletonVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_skeleton_visual();
        visual.background =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.background);
        visual.background_disabled = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.background_disabled);
        visual.highlight = first_rgba_attribute(metadata, &["highlight_color", "border_color"])
            .unwrap_or(visual.highlight);
        visual.highlight_disabled = first_rgba_attribute(
            metadata,
            &["disabled_highlight_color", "disabled_border_color"],
        )
        .unwrap_or(visual.highlight_disabled);
        visual.corner_radius = metric_attribute(metadata, "corner_radius")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.corner_radius);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.border_width);
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_skeleton_visual() -> &'static SkeletonVisual {
    static VISUAL: OnceLock<SkeletonVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        SkeletonVisual {
            background: tokens.palette.surface[3],
            background_disabled: tokens.palette.surface_disabled,
            highlight: tokens.palette.border,
            highlight_disabled: tokens.palette.border_disabled,
            corner_radius: tokens.controls.small_radius,
            border_width: tokens.controls.border_width,
            min_frame_extent: tokens.controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn skeleton_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_skeleton)
}

pub(super) fn skeleton_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_skeleton)
}

pub(super) fn skeleton_suppresses_owner_surface(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_skeleton)
}

pub(super) fn skeleton_render_commands(
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
    if !is_skeleton(metadata) {
        return Vec::new();
    }
    let visual = SkeletonVisual::resolve(metadata);
    let frame = pixel_aligned_frame(frame);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }

    let state = SkeletonRenderState::resolve(metadata, state_flags, component_state);
    vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        background_color(&state, &visual),
        Some(highlight_color(&state, &visual)),
        visual.border_width,
        skeleton_corner_radius(frame, skeleton_variant(metadata), &visual),
        &state,
        opacity,
    )]
}

#[derive(Clone, Copy)]
struct SkeletonRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl SkeletonRenderState {
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
}

#[derive(Clone, Copy)]
enum SkeletonVariant {
    Rounded,
    Rectangular,
    Circular,
}

fn is_skeleton(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "Skeleton"
}

fn skeleton_variant(metadata: &UiTemplateNodeMetadata) -> SkeletonVariant {
    match string_attribute(metadata, "variant") {
        Some(variant) if variant.eq_ignore_ascii_case("rectangular") => {
            SkeletonVariant::Rectangular
        }
        Some(variant) if variant.eq_ignore_ascii_case("circular") => SkeletonVariant::Circular,
        _ => SkeletonVariant::Rounded,
    }
}

fn skeleton_corner_radius(
    frame: UiFrame,
    variant: SkeletonVariant,
    visual: &SkeletonVisual,
) -> f32 {
    match variant {
        SkeletonVariant::Rectangular => 0.0,
        SkeletonVariant::Rounded => visual
            .corner_radius
            .min(frame.width.min(frame.height) * 0.5),
        SkeletonVariant::Circular => frame.width.min(frame.height) * 0.5,
    }
}

fn background_color(state: &SkeletonRenderState, visual: &SkeletonVisual) -> UiRgbaColor {
    if state.disabled() {
        visual.background_disabled
    } else {
        visual.background
    }
}

fn highlight_color(state: &SkeletonRenderState, visual: &SkeletonVisual) -> UiRgbaColor {
    if state.disabled() {
        visual.highlight_disabled
    } else {
        visual.highlight
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
    state: &SkeletonRenderState,
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
