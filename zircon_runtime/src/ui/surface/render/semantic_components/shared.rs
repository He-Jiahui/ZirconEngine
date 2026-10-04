use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{
        UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiRichTextFormat, UiTextAlign,
        UiTextWrap,
    },
    tree::UiTemplateNodeMetadata,
};

use super::super::clipping::intersect_clip_frame;

pub(super) const ACCENT: &str = "#8b7cff";
pub(super) const INFO: &str = "#5da9ff";
pub(super) const SUCCESS: &str = "#55d6a0";
pub(super) const WARNING: &str = "#f4bd62";
pub(super) const ERROR: &str = "#ff7d8a";
pub(super) const TEXT_PRIMARY: &str = "#f4f7fb";
pub(super) const TEXT_SECONDARY: &str = "#a9b4c5";
pub(super) const TEXT_MUTED: &str = "#718096";
pub(super) const SURFACE_INSET: &str = "#11151d";
pub(super) const SURFACE_SELECTED: &str = "#26344a";
pub(super) const BORDER: &str = "#2a3342";

pub(super) fn valid_frame(frame: UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(super) fn component_matches(metadata: &UiTemplateNodeMetadata, candidates: &[&str]) -> bool {
    let matches = |value: &str| {
        let normalized = value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>();
        candidates.iter().any(|candidate| {
            candidate
                .chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
                == normalized
        })
    };
    matches(&metadata.component)
        || metadata
            .attributes
            .get("component_role")
            .and_then(Value::as_str)
            .is_some_and(matches)
}

pub(super) fn text_attribute(metadata: &UiTemplateNodeMetadata, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| metadata.attributes.get(*key).and_then(scalar_text))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn string_array(metadata: &UiTemplateNodeMetadata, key: &str) -> Vec<String> {
    metadata
        .attributes
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(scalar_text)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Applies a caller-provided collection window after the retained layout has
/// established how many rows fit. The source keeps data and window state in
/// props; this painter only projects the currently materialized slice.
pub(super) fn collection_window(
    metadata: &UiTemplateNodeMetadata,
    key: &str,
    physical_capacity: usize,
) -> Vec<String> {
    let start = number_attribute(metadata, "viewport_start")
        .or_else(|| number_attribute(metadata, "visible_start"))
        .unwrap_or_default()
        .max(0.0) as usize;
    let requested = number_attribute(metadata, "visible_limit")
        .or_else(|| number_attribute(metadata, "viewport_count"))
        .map(|value| value.max(0.0) as usize)
        .unwrap_or(physical_capacity);
    string_array(metadata, key)
        .into_iter()
        .skip(start)
        .take(requested.min(physical_capacity))
        .collect()
}

pub(super) fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(|value| match value {
        Value::Boolean(value) => Some(*value),
        Value::String(value) => match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        },
        _ => None,
    })
}

pub(super) fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata
        .attributes
        .get(key)
        .or_else(|| metadata.style_overrides.get(key))
        .and_then(|value| match value {
            Value::Integer(value) => Some(*value as f32),
            Value::Float(value) if value.is_finite() => Some(*value as f32),
            _ => None,
        })
        .filter(|value| value.is_finite())
}

pub(super) fn normalized_state(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<String> {
    text_attribute(metadata, &[key]).map(|value| value.to_ascii_lowercase())
}

pub(super) fn text_color(base_style: &UiResolvedStyle) -> String {
    base_style
        .foreground_color
        .clone()
        .unwrap_or_else(|| TEXT_PRIMARY.to_string())
}

pub(super) fn clipped(clip_frame: Option<UiFrame>, frame: UiFrame) -> Option<UiFrame> {
    intersect_clip_frame(clip_frame, frame)
}

pub(super) fn quad(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    color: impl Into<String>,
    border: Option<String>,
    border_width: f32,
    corner_radius: f32,
    base_style: &UiResolvedStyle,
    opacity: f32,
) -> UiRenderCommand {
    let mut style = base_style.clone();
    style.background_color = Some(color.into());
    style.border_color = border;
    style.border_width = border_width.max(0.0);
    style.corner_radius = corner_radius.max(0.0);
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame: clipped(clip_frame, frame),
        z_index,
        style,
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

pub(super) fn text(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    value: impl Into<String>,
    color: impl Into<String>,
    font_size: f32,
    base_style: &UiResolvedStyle,
    opacity: f32,
) -> UiRenderCommand {
    text_with_presentation(
        node_id,
        frame,
        clip_frame,
        z_index,
        value,
        color,
        font_size,
        base_style.text_align,
        UiRichTextFormat::Plain,
        base_style,
        opacity,
    )
}

pub(super) fn text_aligned(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    value: impl Into<String>,
    color: impl Into<String>,
    font_size: f32,
    alignment: UiTextAlign,
    base_style: &UiResolvedStyle,
    opacity: f32,
) -> UiRenderCommand {
    text_with_presentation(
        node_id,
        frame,
        clip_frame,
        z_index,
        value,
        color,
        font_size,
        alignment,
        UiRichTextFormat::Plain,
        base_style,
        opacity,
    )
}

pub(super) fn rich_text(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    value: impl Into<String>,
    color: impl Into<String>,
    font_size: f32,
    base_style: &UiResolvedStyle,
    opacity: f32,
) -> UiRenderCommand {
    text_with_presentation(
        node_id,
        frame,
        clip_frame,
        z_index,
        value,
        color,
        font_size,
        base_style.text_align,
        UiRichTextFormat::MarkdownInlineV1,
        base_style,
        opacity,
    )
}

fn text_with_presentation(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    value: impl Into<String>,
    color: impl Into<String>,
    font_size: f32,
    alignment: UiTextAlign,
    rich_text_format: UiRichTextFormat,
    base_style: &UiResolvedStyle,
    opacity: f32,
) -> UiRenderCommand {
    let font_size = font_size.max(1.0);
    let mut style = base_style.clone();
    style.background_color = None;
    style.border_color = None;
    style.border_width = 0.0;
    style.corner_radius = 0.0;
    style.foreground_color = Some(color.into());
    style.font_size = font_size;
    style.line_height = base_style.line_height.max(font_size);
    style.text_align = alignment;
    style.wrap = UiTextWrap::Word;
    style.rich_text_format = rich_text_format;
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: clipped(clip_frame, frame),
        z_index,
        style,
        text_layout: None,
        text: Some(value.into()),
        image: None,
        opacity,
    }
}

pub(super) fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Integer(value) => Some(value.to_string()),
        Value::Float(value) => Some(value.to_string()),
        Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/shared.rs"]
mod tests;
