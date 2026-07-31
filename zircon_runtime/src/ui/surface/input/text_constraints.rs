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
        if normalized_constraint_matches(value, &["digits", "digit", "numericdigits"]) {
            Self::Digits
        } else if normalized_constraint_matches(value, &["number", "numeric", "decimal"]) {
            Self::Number
        } else if normalized_constraint_matches(value, &["ascii"]) {
            Self::Ascii
        } else if normalized_constraint_matches(value, &["alphanumeric", "alnum"]) {
            Self::Alphanumeric
        } else {
            Self::Any
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
        filter: metadata
            .attributes
            .get("input_filter")
            .or_else(|| metadata.attributes.get("text_filter"))
            .and_then(toml::Value::as_str)
            .map(TextInputFilter::from_token)
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

fn normalized_constraint_matches(value: &str, expected: &[&str]) -> bool {
    expected.iter().any(|expected| {
        value
            .bytes()
            .filter(|byte| !matches!(byte, b'_' | b'-') && !byte.is_ascii_whitespace())
            .map(|byte| byte.to_ascii_lowercase())
            .eq(expected.bytes())
    })
}

fn clamp_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
