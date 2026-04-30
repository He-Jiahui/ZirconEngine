use toml::Value;

use crate::ui::tree::UiTemplateNodeMetadata;

use super::{
    UiRenderCommandKind, UiResolvedStyle, UiTextAlign, UiTextRenderMode, UiTextWrap,
    UiVisualAssetRef,
};

pub(super) fn resolve_command_kind(
    style: &UiResolvedStyle,
    text: Option<&String>,
    image: Option<&UiVisualAssetRef>,
) -> UiRenderCommandKind {
    if style.background_color.is_some()
        || style.border_color.is_some()
        || style.border_width > 0.0
        || style.corner_radius > 0.0
    {
        UiRenderCommandKind::Quad
    } else if text.is_some() {
        UiRenderCommandKind::Text
    } else if image.is_some() {
        UiRenderCommandKind::Image
    } else {
        UiRenderCommandKind::Group
    }
}

pub(super) fn resolve_style(metadata: Option<&UiTemplateNodeMetadata>) -> UiResolvedStyle {
    let font_size = resolve_table_number(metadata, "font", "size")
        .or_else(|| resolve_number_attribute(metadata, "font_size"))
        .unwrap_or(UiResolvedStyle::DEFAULT_FONT_SIZE);
    UiResolvedStyle {
        background_color: resolve_color_attribute(metadata, "background"),
        foreground_color: resolve_color_attribute(metadata, "foreground"),
        border_color: resolve_color_attribute(metadata, "border"),
        border_width: resolve_table_number(metadata, "border", "width")
            .or_else(|| resolve_number_attribute(metadata, "border_width"))
            .unwrap_or(0.0),
        corner_radius: resolve_table_number(metadata, "border", "radius")
            .or_else(|| resolve_number_attribute(metadata, "radius"))
            .or_else(|| resolve_number_attribute(metadata, "corner_radius"))
            .unwrap_or(0.0),
        font: resolve_table_string(metadata, "font", "asset")
            .or_else(|| resolve_table_string(metadata, "font", "path"))
            .or_else(|| resolve_string_attribute(metadata, "font"))
            .map(str::to_string),
        font_family: resolve_table_string(metadata, "font", "family")
            .or_else(|| resolve_string_attribute(metadata, "font_family"))
            .map(str::to_string),
        font_size,
        line_height: resolve_table_number(metadata, "font", "line_height")
            .or_else(|| resolve_number_attribute(metadata, "line_height"))
            .unwrap_or_else(|| UiResolvedStyle::default_line_height(font_size)),
        text_align: resolve_table_string(metadata, "font", "align")
            .or_else(|| resolve_string_attribute(metadata, "text_align"))
            .and_then(parse_text_align)
            .unwrap_or_default(),
        wrap: resolve_table_string(metadata, "font", "wrap")
            .or_else(|| resolve_string_attribute(metadata, "wrap"))
            .and_then(parse_text_wrap)
            .unwrap_or_default(),
        text_render_mode: resolve_table_string(metadata, "font", "render_mode")
            .or_else(|| resolve_string_attribute(metadata, "text_render_mode"))
            .and_then(parse_text_render_mode)
            .unwrap_or_default(),
    }
}

pub(super) fn resolve_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    resolve_non_empty_string_attribute(metadata, "text")
        .or_else(|| resolve_non_empty_string_attribute(metadata, "label"))
        .map(str::to_string)
}

pub(super) fn resolve_image(metadata: Option<&UiTemplateNodeMetadata>) -> Option<UiVisualAssetRef> {
    resolve_string_attribute(metadata, "icon")
        .map(|icon| UiVisualAssetRef::Icon(icon.to_string()))
        .or_else(|| {
            resolve_string_attribute(metadata, "image")
                .map(|image| UiVisualAssetRef::Image(image.to_string()))
        })
}

pub(super) fn resolve_opacity(metadata: Option<&UiTemplateNodeMetadata>) -> f32 {
    resolve_number_attribute(metadata, "opacity")
        .unwrap_or(1.0)
        .clamp(0.0, 1.0)
}

fn resolve_string_attribute<'a>(
    metadata: Option<&'a UiTemplateNodeMetadata>,
    key: &str,
) -> Option<&'a str> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(Value::as_str)
}

fn resolve_non_empty_string_attribute<'a>(
    metadata: Option<&'a UiTemplateNodeMetadata>,
    key: &str,
) -> Option<&'a str> {
    resolve_string_attribute(metadata, key).filter(|value| !value.is_empty())
}

fn resolve_number_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<f32> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(value_as_f32)
}

fn resolve_table_number(
    metadata: Option<&UiTemplateNodeMetadata>,
    table_key: &str,
    value_key: &str,
) -> Option<f32> {
    metadata
        .and_then(|metadata| metadata.attributes.get(table_key))
        .and_then(Value::as_table)
        .and_then(|table| table.get(value_key))
        .and_then(value_as_f32)
}

fn resolve_table_string<'a>(
    metadata: Option<&'a UiTemplateNodeMetadata>,
    table_key: &str,
    value_key: &str,
) -> Option<&'a str> {
    metadata
        .and_then(|metadata| metadata.attributes.get(table_key))
        .and_then(Value::as_table)
        .and_then(|table| table.get(value_key))
        .and_then(Value::as_str)
}

fn resolve_color_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<String> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(resolve_color_value)
}

fn resolve_color_value(value: &Value) -> Option<String> {
    match value {
        Value::String(color) => Some(color.clone()),
        Value::Table(table) => table
            .get("color")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}

fn parse_text_align(value: &str) -> Option<UiTextAlign> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" | "start" => Some(UiTextAlign::Left),
        "center" | "middle" => Some(UiTextAlign::Center),
        "right" | "end" => Some(UiTextAlign::Right),
        _ => None,
    }
}

fn parse_text_wrap(value: &str) -> Option<UiTextWrap> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" | "off" | "nowrap" => Some(UiTextWrap::None),
        "word" | "normal" => Some(UiTextWrap::Word),
        "glyph" | "char" | "character" => Some(UiTextWrap::Glyph),
        _ => None,
    }
}

fn parse_text_render_mode(value: &str) -> Option<UiTextRenderMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" | "default" => Some(UiTextRenderMode::Auto),
        "native" | "glyphon" => Some(UiTextRenderMode::Native),
        "sdf" => Some(UiTextRenderMode::Sdf),
        _ => None,
    }
}
