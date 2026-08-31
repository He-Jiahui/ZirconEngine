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
struct DividerVisual {
    separator: UiRgbaColor,
    separator_disabled: UiRgbaColor,
    thickness: f32,
    inset: f32,
    min_frame_extent: f32,
}

impl DividerVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_divider_visual();
        visual.separator = first_rgba_attribute(metadata, &["separator_color", "color"])
            .unwrap_or(visual.separator);
        visual.separator_disabled = first_rgba_attribute(metadata, &["disabled_separator_color"])
            .unwrap_or(visual.separator_disabled);
        visual.thickness = metric_attribute(metadata, "thickness")
            .or_else(|| metric_attribute(metadata, "border_width"))
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.thickness);
        visual.inset = metric_attribute(metadata, "inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.inset);
        visual.min_frame_extent = visual
            .thickness
            .min(visual.min_frame_extent)
            .max(f32::EPSILON);
        visual
    }
}

fn default_divider_visual() -> &'static DividerVisual {
    static VISUAL: OnceLock<DividerVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        DividerVisual {
            separator: tokens.palette.separator_soft,
            separator_disabled: tokens.palette.border_disabled,
            thickness: tokens.controls.border_width,
            inset: tokens.density.gap_medium,
            min_frame_extent: tokens.controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn divider_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_divider)
}

pub(super) fn divider_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_divider)
}

pub(super) fn divider_suppresses_owner_surface(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_divider)
}

pub(super) fn divider_render_commands(
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
    if !is_divider(metadata) {
        return Vec::new();
    }
    let visual = DividerVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }

    let state = DividerRenderState::resolve(metadata, state_flags, component_state);
    let line = match orientation(metadata) {
        DividerOrientation::Horizontal => horizontal_divider_frame(frame, metadata, &visual),
        DividerOrientation::Vertical => vertical_divider_frame(frame, metadata, &visual),
    };
    if line.width <= 0.0 || line.height <= 0.0 {
        return Vec::new();
    }
    vec![quad_command(
        node_id,
        line,
        clip_frame,
        z_index.saturating_add(1),
        separator_color(&state, &visual),
        &state,
        opacity,
    )]
}

#[derive(Clone, Copy)]
struct DividerRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl DividerRenderState {
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
enum DividerOrientation {
    Horizontal,
    Vertical,
}

fn is_divider(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(metadata.component.as_str(), "Divider" | "Separator")
}

fn orientation(metadata: &UiTemplateNodeMetadata) -> DividerOrientation {
    string_attribute(metadata, "orientation")
        .or_else(|| string_attribute(metadata, "direction"))
        .is_some_and(|orientation| orientation.eq_ignore_ascii_case("vertical"))
        .then_some(DividerOrientation::Vertical)
        .unwrap_or(DividerOrientation::Horizontal)
}

fn horizontal_divider_frame(
    frame: UiFrame,
    metadata: &UiTemplateNodeMetadata,
    visual: &DividerVisual,
) -> UiFrame {
    let thickness = visual.thickness.min(frame.height);
    let (leading, trailing) = divider_insets(frame.width, metadata, visual);
    UiFrame::new(
        frame.x + leading,
        frame.y + (frame.height - thickness) * 0.5,
        (frame.width - leading - trailing).max(0.0),
        thickness,
    )
}

fn vertical_divider_frame(
    frame: UiFrame,
    metadata: &UiTemplateNodeMetadata,
    visual: &DividerVisual,
) -> UiFrame {
    let thickness = visual.thickness.min(frame.width);
    let (leading, trailing) = divider_insets(frame.height, metadata, visual);
    UiFrame::new(
        frame.x + (frame.width - thickness) * 0.5,
        frame.y + leading,
        thickness,
        (frame.height - leading - trailing).max(0.0),
    )
}

fn divider_insets(
    available_extent: f32,
    metadata: &UiTemplateNodeMetadata,
    visual: &DividerVisual,
) -> (f32, f32) {
    let inset = visual.inset.min(available_extent * 0.5);
    match string_attribute(metadata, "variant") {
        Some(variant) if variant.eq_ignore_ascii_case("middle") => (inset, inset),
        Some(variant) if variant.eq_ignore_ascii_case("inset") => (inset, 0.0),
        _ => (0.0, 0.0),
    }
}

fn separator_color(state: &DividerRenderState, visual: &DividerVisual) -> UiRgbaColor {
    if state.disabled() {
        visual.separator_disabled
    } else {
        visual.separator
    }
}

fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: UiRgbaColor,
    state: &DividerRenderState,
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
