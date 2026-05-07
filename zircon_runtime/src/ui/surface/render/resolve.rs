use toml::Value;

use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiRenderCommandKind, UiResolvedStyle, UiTextAlign, UiTextCaret,
    UiTextComposition, UiTextDirection, UiTextOverflow, UiTextRange, UiTextRenderMode,
    UiTextSelection, UiTextWrap, UiVisualAssetRef,
};
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

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
    } else if image.is_some() {
        UiRenderCommandKind::Image
    } else if text.is_some() {
        UiRenderCommandKind::Text
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
        text_direction: resolve_table_string(metadata, "font", "direction")
            .or_else(|| resolve_string_attribute(metadata, "text_direction"))
            .and_then(parse_text_direction)
            .unwrap_or_default(),
        text_overflow: resolve_table_string(metadata, "font", "overflow")
            .or_else(|| resolve_string_attribute(metadata, "text_overflow"))
            .and_then(parse_text_overflow)
            .unwrap_or_default(),
        rich_text: resolve_bool_attribute(metadata, "rich_text").unwrap_or(false),
        text_render_mode: resolve_table_string(metadata, "font", "render_mode")
            .or_else(|| resolve_string_attribute(metadata, "text_render_mode"))
            .and_then(parse_text_render_mode)
            .unwrap_or_default(),
    }
}

pub(super) fn resolve_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    if let Some(label) = resolve_visible_label_text(metadata) {
        return Some(label);
    }

    if let Some(text) = resolve_non_empty_string_attribute(metadata, "text") {
        return Some(text.to_string());
    }

    if icon_button_label_is_accessibility_only(metadata) {
        return None;
    }

    resolve_control_fallback_text(metadata)
}

pub(super) fn resolve_image(metadata: Option<&UiTemplateNodeMetadata>) -> Option<UiVisualAssetRef> {
    resolve_string_attribute(metadata, "icon")
        .map(|icon| UiVisualAssetRef::Icon(icon.to_string()))
        .or_else(|| {
            resolve_string_attribute(metadata, "image")
                .map(|image| UiVisualAssetRef::Image(image.to_string()))
        })
        .or_else(|| {
            resolve_string_attribute(metadata, "source")
                .map(|source| UiVisualAssetRef::Image(source.to_string()))
        })
}

pub(super) fn resolve_opacity(metadata: Option<&UiTemplateNodeMetadata>) -> f32 {
    resolve_number_attribute(metadata, "opacity")
        .unwrap_or(1.0)
        .clamp(0.0, 1.0)
}

pub(super) fn resolve_editable_text_state(
    metadata: Option<&UiTemplateNodeMetadata>,
    visible_text: Option<&str>,
) -> Option<UiEditableTextState> {
    let metadata = metadata?;
    if !is_editable_text_component(metadata) {
        return None;
    }
    let text = resolve_editable_text_value(metadata, visible_text);
    let caret = UiTextCaret {
        offset: clamp_text_boundary(
            &text,
            resolve_usize_attribute(Some(metadata), "caret_offset").unwrap_or(text.len()),
        ),
        affinity: Default::default(),
    };
    let selection = resolve_selection(metadata, &text);
    let composition = resolve_composition(metadata, &text);
    Some(UiEditableTextState {
        text,
        caret,
        selection,
        composition,
        read_only: resolve_bool_attribute(Some(metadata), "read_only")
            .or_else(|| resolve_bool_attribute(Some(metadata), "input_read_only"))
            .unwrap_or(false),
    })
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

fn resolve_visible_value_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    let value = metadata.and_then(|metadata| metadata.attributes.get("value"))?;
    resolve_scalar_text(value)
}

fn icon_button_label_is_accessibility_only(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    metadata.component == "IconButton"
        && (["icon", "image", "source"]
            .iter()
            .any(|key| metadata.attributes.contains_key(*key))
            || resolve_number_attribute(Some(metadata), "layout_icon_size")
                .is_some_and(|size| size > 0.0))
}

fn resolve_control_fallback_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    if !supports_visible_value_fallback(metadata) {
        return None;
    }

    resolve_visible_value_text(metadata)
        .or_else(|| resolve_non_empty_string_attribute(metadata, "value_text").map(str::to_string))
        .or_else(|| resolve_non_empty_string_attribute(metadata, "placeholder").map(str::to_string))
        .or_else(|| resolve_first_option_text(metadata))
}

fn resolve_visible_label_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    if icon_button_label_is_accessibility_only(metadata) {
        return None;
    }
    if !supports_visible_label(metadata) {
        return None;
    }
    resolve_non_empty_string_attribute(metadata, "label").map(str::to_string)
}

fn supports_visible_label(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };

    matches!(
        metadata.component.as_str(),
        "Button"
            | "ToggleButton"
            | "Checkbox"
            | "ComboBox"
            | "MenuItem"
            | "Tab"
            | "TableRow"
            | "ListRow"
            | "Dropdown"
            | "Radio"
            | "SegmentedControl"
    )
}

fn supports_visible_value_fallback(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };

    matches!(
        metadata.component.as_str(),
        "Button"
            | "ToggleButton"
            | "Checkbox"
            | "InputField"
            | "TextField"
            | "ComboBox"
            | "RangeField"
            | "NumberField"
            | "Switch"
            | "ContextActionMenu"
            | "MenuItem"
            | "Tab"
            | "TableRow"
            | "Label"
            | "ListRow"
            | "Dropdown"
            | "Radio"
            | "SegmentedControl"
            | "ColorField"
            | "Vector2Field"
            | "Vector3Field"
            | "Vector4Field"
    )
}

fn is_editable_text_component(metadata: &UiTemplateNodeMetadata) -> bool {
    resolve_bool_attribute(Some(metadata), "editable_text").unwrap_or(false)
        || matches!(
            metadata.component.as_str(),
            "InputField" | "TextField" | "LineEdit" | "TextEdit" | "NumberField"
        )
}

fn resolve_editable_text_value(
    metadata: &UiTemplateNodeMetadata,
    visible_text: Option<&str>,
) -> String {
    metadata
        .attributes
        .get("value")
        .and_then(resolve_scalar_text)
        .or_else(|| {
            resolve_non_empty_string_attribute(Some(metadata), "value_text").map(str::to_string)
        })
        .or_else(|| resolve_non_empty_string_attribute(Some(metadata), "text").map(str::to_string))
        .or_else(|| visible_text.map(str::to_string))
        .unwrap_or_default()
}

fn resolve_selection(metadata: &UiTemplateNodeMetadata, text: &str) -> Option<UiTextSelection> {
    let anchor = resolve_usize_attribute(Some(metadata), "selection_anchor")?;
    let focus = resolve_usize_attribute(Some(metadata), "selection_focus")?;
    Some(UiTextSelection {
        anchor: clamp_text_boundary(text, anchor),
        focus: clamp_text_boundary(text, focus),
    })
}

fn resolve_composition(metadata: &UiTemplateNodeMetadata, text: &str) -> Option<UiTextComposition> {
    let start = resolve_usize_attribute(Some(metadata), "composition_start")?;
    let end = resolve_usize_attribute(Some(metadata), "composition_end")?;
    let composition_text = resolve_string_attribute(Some(metadata), "composition_text")?;
    Some(UiTextComposition {
        range: UiTextRange {
            start: clamp_text_boundary(text, start),
            end: clamp_text_boundary(text, end),
        },
        text: composition_text.to_string(),
        restore_text: resolve_string_attribute(Some(metadata), "composition_restore_text")
            .map(str::to_string),
    })
}

fn resolve_first_option_text(metadata: Option<&UiTemplateNodeMetadata>) -> Option<String> {
    metadata
        .and_then(|metadata| metadata.attributes.get("options"))
        .and_then(Value::as_array)
        .and_then(|options| options.iter().find_map(resolve_option_text))
}

fn resolve_option_text(value: &Value) -> Option<String> {
    match value {
        Value::Table(table) => ["text", "label", "value", "name", "title"]
            .iter()
            .filter_map(|key| table.get(*key))
            .find_map(resolve_scalar_text),
        scalar => resolve_scalar_text(scalar),
    }
}

fn resolve_scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) if !value.is_empty() => Some(value.clone()),
        Value::Integer(value) => Some(value.to_string()),
        Value::Float(value) => Some(value.to_string()),
        Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}

fn resolve_number_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<f32> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(value_as_f32)
}

fn resolve_usize_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<usize> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(|value| match value {
            Value::Integer(value) => (*value >= 0).then_some(*value as usize),
            Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
            _ => None,
        })
}

fn resolve_bool_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<bool> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(Value::as_bool)
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

fn clamp_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
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

fn parse_text_direction(value: &str) -> Option<UiTextDirection> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" | "default" => Some(UiTextDirection::Auto),
        "ltr" | "left_to_right" | "left-to-right" => Some(UiTextDirection::LeftToRight),
        "rtl" | "right_to_left" | "right-to-left" => Some(UiTextDirection::RightToLeft),
        "mixed" => Some(UiTextDirection::Mixed),
        _ => None,
    }
}

fn parse_text_overflow(value: &str) -> Option<UiTextOverflow> {
    match value.trim().to_ascii_lowercase().as_str() {
        "clip" | "clipped" => Some(UiTextOverflow::Clip),
        "ellipsis" | "truncate" => Some(UiTextOverflow::Ellipsis),
        _ => None,
    }
}
