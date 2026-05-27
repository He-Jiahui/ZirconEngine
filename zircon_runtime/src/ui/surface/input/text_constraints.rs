use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, surface::UiTextRange, tree::UiTemplateNodeMetadata,
};

use super::super::surface::UiSurface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextInputConstraints {
    max_graphemes: Option<usize>,
    filter: TextInputFilter,
    multiline: bool,
}

impl Default for TextInputConstraints {
    fn default() -> Self {
        Self {
            max_graphemes: None,
            filter: TextInputFilter::Any,
            multiline: true,
        }
    }
}

impl TextInputConstraints {
    pub(crate) fn allows_multiline(self) -> bool {
        self.multiline
    }

    pub(crate) fn sanitize_replacement(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
    ) -> String {
        let filtered = self.filter_text(replacement);
        let Some(max_graphemes) = self.max_graphemes else {
            return filtered;
        };
        let retained = retained_grapheme_count(current_text, replaced_range);
        let available = max_graphemes.saturating_sub(retained);
        take_graphemes(&filtered, available)
    }

    fn filter_text(self, text: &str) -> String {
        text.chars()
            .filter(|ch| self.multiline || !matches!(ch, '\r' | '\n'))
            .filter(|ch| self.filter.accepts(*ch))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TextInputFilter {
    #[default]
    Any,
    Digits,
    Number,
    Ascii,
    Alphanumeric,
}

impl TextInputFilter {
    fn from_token(value: &str) -> Self {
        match normalize_constraint_token(value).as_str() {
            "digits" | "digit" | "numericdigits" => Self::Digits,
            "number" | "numeric" | "decimal" => Self::Number,
            "ascii" => Self::Ascii,
            "alphanumeric" | "alnum" => Self::Alphanumeric,
            _ => Self::Any,
        }
    }

    fn accepts(self, ch: char) -> bool {
        match self {
            Self::Any => true,
            Self::Digits => ch.is_ascii_digit(),
            Self::Number => ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+'),
            Self::Ascii => ch.is_ascii(),
            Self::Alphanumeric => ch.is_alphanumeric(),
        }
    }
}

pub(crate) fn text_input_constraints_for_node(
    surface: &UiSurface,
    target: UiNodeId,
) -> TextInputConstraints {
    let Some(metadata) = surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref())
    else {
        return TextInputConstraints::default();
    };
    TextInputConstraints {
        max_graphemes: usize_attribute(metadata, "max_graphemes")
            .or_else(|| usize_attribute(metadata, "max_chars"))
            .or_else(|| usize_attribute(metadata, "max_length")),
        filter: string_attribute(metadata, "input_filter")
            .or_else(|| string_attribute(metadata, "text_filter"))
            .map(|value| TextInputFilter::from_token(value.as_str()))
            .unwrap_or_default(),
        multiline: bool_attribute(metadata, "multiline").unwrap_or(true),
    }
}

fn retained_grapheme_count(text: &str, replaced_range: UiTextRange) -> usize {
    let start = clamp_text_boundary(text, replaced_range.start);
    let end = clamp_text_boundary(text, replaced_range.end).max(start);
    text[..start].graphemes(true).count() + text[end..].graphemes(true).count()
}

fn take_graphemes(text: &str, max_graphemes: usize) -> String {
    text.graphemes(true).take(max_graphemes).collect()
}

fn string_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<String> {
    metadata.attributes.get(key).and_then(|value| match value {
        toml::Value::String(value) => Some(value.clone()),
        toml::Value::Integer(value) => Some(value.to_string()),
        toml::Value::Float(value) => Some(value.to_string()),
        toml::Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    })
}

fn usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    metadata.attributes.get(key).and_then(|value| match value {
        toml::Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        toml::Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    })
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(toml::Value::as_bool)
}

fn normalize_constraint_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| *ch != '_' && *ch != '-' && !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

fn clamp_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
