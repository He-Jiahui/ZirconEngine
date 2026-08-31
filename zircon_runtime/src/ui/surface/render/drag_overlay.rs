use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

#[derive(Clone, Copy, Debug)]
struct DragOverlayVisual {
    allowed_surface: UiRgbaColor,
    blocked_surface: UiRgbaColor,
    allowed_accent: UiRgbaColor,
    blocked_accent: UiRgbaColor,
    text: UiRgbaColor,
    corner_radius: f32,
    border_width: f32,
    icon_left: f32,
    icon_size: f32,
    text_icon_gap: f32,
    text_right_inset: f32,
    font_size: f32,
    line_height: f32,
    indicator_thickness: f32,
    cursor_offset: f32,
    min_frame_extent: f32,
}

impl DragOverlayVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_drag_overlay_visual();
        visual.allowed_surface =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.allowed_surface);
        visual.blocked_surface = first_rgba_attribute(metadata, &["blocked_background_color"])
            .unwrap_or(visual.blocked_surface);
        visual.allowed_accent = first_rgba_attribute(metadata, &["border_color", "accent_color"])
            .unwrap_or(visual.allowed_accent);
        visual.blocked_accent = first_rgba_attribute(metadata, &["blocked_border_color"])
            .unwrap_or(visual.blocked_accent);
        visual.text = first_rgba_attribute(metadata, &["foreground_color", "text_color"])
            .unwrap_or(visual.text);
        visual.corner_radius = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.corner_radius);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.icon_left = metric_attribute(metadata, "icon_left_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.icon_left);
        visual.icon_size = metric_attribute(metadata, "layout_icon_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.icon_size);
        visual.text_right_inset = metric_attribute(metadata, "text_right_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.text_right_inset);
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
        visual.indicator_thickness = metric_attribute(metadata, "indicator_thickness")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.indicator_thickness);
        visual.cursor_offset = metric_attribute(metadata, "cursor_offset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.cursor_offset);
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_drag_overlay_visual() -> &'static DragOverlayVisual {
    static VISUAL: OnceLock<DragOverlayVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        DragOverlayVisual {
            allowed_surface: colors.accent_soft,
            blocked_surface: colors.error_container,
            allowed_accent: colors.accent,
            blocked_accent: colors.error,
            text: colors.text_primary,
            corner_radius: controls.control_radius,
            border_width: controls.border_width,
            icon_left: density.gap_large,
            icon_size: controls.dense_height - density.gap_medium - controls.border_width * 2.0,
            text_icon_gap: density.gap_medium,
            text_right_inset: density.gap_large,
            font_size: typography.overlay_size,
            line_height: typography.overlay_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            indicator_thickness: controls.border_width * 2.0,
            cursor_offset: density.gap_large,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn drag_overlay_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_drag_overlay)
}
pub(super) fn drag_overlay_suppresses_owner_image(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_drag_overlay)
}
pub(super) fn drag_overlay_suppresses_owner_surface(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_drag_overlay)
}

pub(super) fn drag_overlay_render_commands(
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
    if !is_drag_overlay(metadata) || !drag_overlay_open(metadata, component_state) {
        return Vec::new();
    }
    let visual = DragOverlayVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let drop_allowed = bool_attribute(metadata, "drop_allowed").unwrap_or(true);
    let state = DragOverlayRenderState::resolve(metadata, state_flags, component_state);
    let preview = preview_frame(metadata, frame, &visual);
    let (surface, accent) = if drop_allowed {
        (visual.allowed_surface, visual.allowed_accent)
    } else {
        (visual.blocked_surface, visual.blocked_accent)
    };
    let mut commands = vec![quad_command(
        node_id,
        preview,
        clip_frame,
        z_index.saturating_add(1),
        surface,
        Some(accent),
        visual.border_width,
        visual.corner_radius,
        state.preview_state,
        opacity,
    )];
    let icon = payload_icon(metadata);
    if let Some(icon) = icon {
        commands.push(image_command(
            node_id,
            UiFrame::new(
                preview.x + visual.icon_left,
                preview.y + (preview.height - visual.icon_size).max(0.0) * 0.5,
                visual.icon_size,
                visual.icon_size,
            ),
            clip_frame,
            z_index.saturating_add(2),
            icon,
            accent,
            state.preview_state,
            opacity,
        ));
    }
    if let Some(label) = preview_label(metadata) {
        let text_left = preview.x
            + if icon.is_some() {
                visual.icon_left + visual.icon_size + visual.text_icon_gap
            } else {
                visual.icon_left
            };
        let text_width =
            (preview.right() - visual.text_right_inset - text_left).max(visual.min_frame_extent);
        commands.push(text_command(
            node_id,
            UiFrame::new(
                text_left,
                preview.y + (preview.height - visual.line_height).max(0.0) * 0.5,
                text_width,
                visual.line_height,
            ),
            clip_frame,
            z_index.saturating_add(3),
            label,
            visual.text,
            visual.font_size,
            visual.line_height,
            state.preview_state,
            opacity,
        ));
    }
    if let Some(indicator) = indicator_frame(metadata, &visual) {
        commands.push(quad_command(
            node_id,
            indicator,
            clip_frame,
            z_index.saturating_add(4),
            accent,
            None,
            0.0,
            visual.border_width,
            UiPainterResolvedState::DropHovered,
            opacity,
        ));
    }
    commands
}

#[derive(Clone, Copy)]
struct DragOverlayRenderState {
    preview_state: UiPainterResolvedState,
}
impl DragOverlayRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let state = UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
            .painter_state();
        Self {
            preview_state: state.resolved_state_for_family(UiPainterFamily::Chrome),
        }
    }
}
fn is_drag_overlay(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "DragOverlay"
}
fn drag_overlay_open(
    metadata: &UiTemplateNodeMetadata,
    component_state: Option<&UiComponentState>,
) -> bool {
    bool_attribute(metadata, "open").unwrap_or(false)
        || bool_attribute(metadata, "dragging").unwrap_or(false)
        || component_state.is_some_and(|state| state.flags.dragging)
}
fn preview_frame(
    metadata: &UiTemplateNodeMetadata,
    fallback: UiFrame,
    visual: &DragOverlayVisual,
) -> UiFrame {
    let width = metric_attribute(metadata, "preview_width")
        .filter(|value| *value > 0.0)
        .unwrap_or(fallback.width);
    let height = metric_attribute(metadata, "preview_height")
        .filter(|value| *value > 0.0)
        .unwrap_or(fallback.height);
    match (
        metric_attribute(metadata, "cursor_x"),
        metric_attribute(metadata, "cursor_y"),
    ) {
        (Some(x), Some(y)) => UiFrame::new(
            x + metric_attribute(metadata, "offset_x")
                .filter(|value| *value >= 0.0)
                .unwrap_or(visual.cursor_offset),
            y + metric_attribute(metadata, "offset_y")
                .filter(|value| *value >= 0.0)
                .unwrap_or(visual.cursor_offset),
            width,
            height,
        ),
        _ => UiFrame::new(fallback.x, fallback.y, width, height),
    }
}
fn indicator_frame(
    metadata: &UiTemplateNodeMetadata,
    visual: &DragOverlayVisual,
) -> Option<UiFrame> {
    let edge = string_attribute(metadata, "drop_indicator_edge").unwrap_or("none");
    if edge == "none" {
        return None;
    }
    let x = metric_attribute(metadata, "drop_target_x")?;
    let y = metric_attribute(metadata, "drop_target_y")?;
    let width = metric_attribute(metadata, "drop_target_width")
        .filter(|value| *value > 0.0)
        .unwrap_or(visual.min_frame_extent);
    let height = metric_attribute(metadata, "drop_target_height")
        .filter(|value| *value > 0.0)
        .unwrap_or(visual.min_frame_extent);
    let t = visual.indicator_thickness;
    match edge {
        "top" => Some(UiFrame::new(x, y, width, t)),
        "bottom" => Some(UiFrame::new(x, y + (height - t).max(0.0), width, t)),
        "left" => Some(UiFrame::new(x, y, t, height)),
        "right" => Some(UiFrame::new(x + (width - t).max(0.0), y, t, height)),
        "inside" => Some(UiFrame::new(x, y, width, height)),
        _ => None,
    }
}
fn preview_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    string_attribute(metadata, "payload_label")
        .filter(|value| !value.is_empty())
        .or_else(|| string_attribute(metadata, "text").filter(|value| !value.is_empty()))
        .or_else(|| {
            string_attribute(metadata, "payload_reference").filter(|value| !value.is_empty())
        })
        .map(ToOwned::to_owned)
}
fn payload_icon(metadata: &UiTemplateNodeMetadata) -> Option<&'static str> {
    match string_attribute(metadata, "payload_kind").unwrap_or("unknown") {
        "asset" => Some("package"),
        "scene-instance" => Some("box"),
        "object" => Some("cube"),
        _ => None,
    }
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
    let encoded = encoded.as_bytes();
    let (r, g, b, a) = match encoded.len() {
        6 => (
            decode_hex_byte(encoded, 0)?,
            decode_hex_byte(encoded, 2)?,
            decode_hex_byte(encoded, 4)?,
            u8::MAX,
        ),
        8 => (
            decode_hex_byte(encoded, 0)?,
            decode_hex_byte(encoded, 2)?,
            decode_hex_byte(encoded, 4)?,
            decode_hex_byte(encoded, 6)?,
        ),
        _ => return None,
    };
    Some(UiRgbaColor::from_u8(r, g, b, a))
}

fn decode_hex_byte(encoded: &[u8], offset: usize) -> Option<u8> {
    let high = decode_hex_digit(*encoded.get(offset)?)?;
    let low = decode_hex_digit(*encoded.get(offset + 1)?)?;
    Some((high << 4) | low)
}

fn decode_hex_digit(encoded: u8) -> Option<u8> {
    match encoded {
        b'0'..=b'9' => Some(encoded - b'0'),
        b'a'..=b'f' => Some(encoded - b'a' + 10),
        b'A'..=b'F' => Some(encoded - b'A' + 10),
        _ => None,
    }
}

fn css_color(color: UiRgbaColor) -> String {
    let [r, g, b, a] = color.to_u8();
    let mut value = if a == u8::MAX {
        format!("{r:02x}{g:02x}{b:02x}")
    } else {
        format!("{r:02x}{g:02x}{b:02x}{a:02x}")
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
    radius: f32,
    state: UiPainterResolvedState,
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
            corner_radius: radius,
            ..UiResolvedStyle::default().with_painter_state(UiPainterFamily::Chrome, state)
        },
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}
fn image_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    icon: &str,
    color: UiRgbaColor,
    state: UiPainterResolvedState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Image,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(css_color(color)),
            ..UiResolvedStyle::default().with_painter_state(UiPainterFamily::Chrome, state)
        },
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon.to_string())),
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
    color: UiRgbaColor,
    font_size: f32,
    line_height: f32,
    state: UiPainterResolvedState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(css_color(color)),
            font_size,
            line_height,
            ..UiResolvedStyle::default().with_painter_state(UiPainterFamily::Chrome, state)
        },
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}

#[cfg(test)]
#[path = "drag_overlay/direct_hex_color_tests.rs"]
mod direct_hex_color_tests;
