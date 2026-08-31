use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    dispatch::{UiImePreeditClause, UiTextByteRange, UiTextInputConstraintReceipt},
    event_ui::UiNodeId,
    surface::UiTextRange,
    tree::UiTemplateNodeMetadata,
};

use super::super::surface::UiSurface;
use crate::ui::text::clamp_grapheme_boundary;

use self::preedit_mapping::{
    TextInputBoundaryMap, remap_preedit_clauses, remap_preedit_cursor_range,
};

mod preedit_mapping;
#[cfg(test)]
mod tests;

const TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES: [&str; 3] = [
    "text_input_grapheme_document_index_count",
    "text_input_grapheme_source_scan_count",
    "text_input_grapheme_source_scan_bytes",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextInputRetainedGraphemeCount {
    DocumentIndex(usize),
    #[default]
    SourceScan,
}

pub(crate) struct SanitizedTextInputReplacement {
    pub(crate) text: String,
    pub(crate) receipt: UiTextInputConstraintReceipt,
}

pub(crate) struct SanitizedTextInputPreedit {
    pub(crate) text: String,
    pub(crate) cursor_range: Option<UiTextByteRange>,
    pub(crate) preedit_clauses: Vec<UiImePreeditClause>,
    pub(crate) receipt: UiTextInputConstraintReceipt,
}

impl SanitizedTextInputReplacement {
    pub(crate) fn receipt_if_changed(&self) -> Option<UiTextInputConstraintReceipt> {
        (!self.receipt.is_empty()).then_some(self.receipt)
    }
}

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
    ) -> SanitizedTextInputReplacement {
        self.sanitize_replacement_with_boundary_map(
            current_text,
            replaced_range,
            replacement,
            TextInputRetainedGraphemeCount::SourceScan,
            None,
        )
    }

    pub(crate) fn sanitize_replacement_with_retained_grapheme_count(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        retained_graphemes: TextInputRetainedGraphemeCount,
    ) -> SanitizedTextInputReplacement {
        self.sanitize_replacement_with_boundary_map(
            current_text,
            replaced_range,
            replacement,
            retained_graphemes,
            None,
        )
    }

    pub(crate) fn sanitize_preedit_replacement(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: &[UiImePreeditClause],
    ) -> SanitizedTextInputPreedit {
        self.sanitize_preedit_replacement_with_retained_grapheme_count(
            current_text,
            replaced_range,
            replacement,
            cursor_range,
            preedit_clauses,
            TextInputRetainedGraphemeCount::SourceScan,
        )
    }

    pub(crate) fn sanitize_preedit_replacement_with_retained_grapheme_count(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: &[UiImePreeditClause],
        retained_graphemes: TextInputRetainedGraphemeCount,
    ) -> SanitizedTextInputPreedit {
        let mut boundary_map = TextInputBoundaryMap::new(cursor_range, preedit_clauses);
        let SanitizedTextInputReplacement {
            text,
            receipt: mut receipt,
        } = self.sanitize_replacement_with_boundary_map(
            current_text,
            replaced_range,
            replacement,
            retained_graphemes,
            Some(&mut boundary_map),
        );
        let cursor_range =
            remap_preedit_cursor_range(&boundary_map, &text, cursor_range, &mut receipt);
        let preedit_clauses = remap_preedit_clauses(&boundary_map, preedit_clauses, &mut receipt);
        SanitizedTextInputPreedit {
            text,
            cursor_range,
            preedit_clauses,
            receipt,
        }
    }

    fn sanitize_replacement_with_boundary_map(
        self,
        current_text: &str,
        replaced_range: UiTextRange,
        replacement: &str,
        retained_graphemes: TextInputRetainedGraphemeCount,
        mut boundary_map: Option<&mut TextInputBoundaryMap>,
    ) -> SanitizedTextInputReplacement {
        let mut receipt = UiTextInputConstraintReceipt::default();
        let mut filtered = String::with_capacity(replacement.len());
        let mut characters = replacement.char_indices().peekable();
        if let Some(boundary_map) = boundary_map.as_deref_mut() {
            boundary_map.record(0, 0);
        }
        while let Some((offset, character)) = characters.next() {
            let character_end = offset + character.len_utf8();
            if !self.multiline && crate::text::is_hard_line_separator(character) {
                receipt.removed_hard_line_count = receipt.removed_hard_line_count.saturating_add(1);
                if let Some(boundary_map) = boundary_map.as_deref_mut() {
                    boundary_map.record(character_end, filtered.len());
                }
                if character == '\r'
                    && characters
                        .peek()
                        .is_some_and(|(_, next_character)| *next_character == '\n')
                {
                    let (line_feed_offset, line_feed) = characters.next().unwrap();
                    if let Some(boundary_map) = boundary_map.as_deref_mut() {
                        boundary_map
                            .record(line_feed_offset + line_feed.len_utf8(), filtered.len());
                    }
                }
                continue;
            }
            if !self.filter.accepts(character) {
                receipt.removed_filter_scalar_count =
                    receipt.removed_filter_scalar_count.saturating_add(1);
                if let Some(boundary_map) = boundary_map.as_deref_mut() {
                    boundary_map.record(character_end, filtered.len());
                }
                continue;
            }
            filtered.push(character);
            if let Some(boundary_map) = boundary_map.as_deref_mut() {
                boundary_map.record(character_end, filtered.len());
            }
        }

        if let Some(max_graphemes) = self.max_graphemes {
            let retained = match retained_graphemes {
                TextInputRetainedGraphemeCount::DocumentIndex(retained) => {
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[0],
                        1
                    );
                    retained
                }
                TextInputRetainedGraphemeCount::SourceScan => {
                    let (retained, scanned_bytes) =
                        retained_grapheme_count_from_source(current_text, replaced_range);
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[1],
                        1
                    );
                    crate::profile_counter!(
                        "runtime",
                        TEXT_INPUT_GRAPHEME_AUTHORITY_COUNTER_NAMES[2],
                        scanned_bytes
                    );
                    retained
                }
            };
            let available = max_graphemes.saturating_sub(retained);
            if let Some((truncate_at, _)) = filtered.grapheme_indices(true).nth(available) {
                filtered.truncate(truncate_at);
                receipt.max_graphemes_truncated = true;
            }
        }
        if let Some(boundary_map) = boundary_map {
            boundary_map.clamp_output(filtered.len());
        }
        SanitizedTextInputReplacement {
            text: filtered,
            receipt,
        }
    }

    pub(crate) const fn requires_retained_grapheme_count(self) -> bool {
        self.max_graphemes.is_some()
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
            Self::Number => ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E'),
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
    let configured_max_graphemes = positive_usize_attribute(metadata, "max_graphemes")
        .or_else(|| positive_usize_attribute(metadata, "max_chars"))
        .or_else(|| positive_usize_attribute(metadata, "max_length"));
    let max_graphemes = if metadata.component == "NumberField" {
        Some(
            configured_max_graphemes
                .unwrap_or(super::number_field::MVP_MAX_NUMBER_FIELD_EDIT_BYTES)
                .min(super::number_field::MVP_MAX_NUMBER_FIELD_EDIT_BYTES),
        )
    } else {
        configured_max_graphemes
    };
    TextInputConstraints {
        max_graphemes,
        filter: metadata
            .attributes
            .get("input_filter")
            .or_else(|| metadata.attributes.get("text_filter"))
            .and_then(toml::Value::as_str)
            .map(TextInputFilter::from_token)
            .unwrap_or_else(|| {
                if metadata.component == "NumberField" {
                    TextInputFilter::Number
                } else {
                    TextInputFilter::Any
                }
            }),
        multiline: bool_attribute(metadata, "multiline")
            .unwrap_or(metadata.component != "NumberField"),
    }
}

fn retained_grapheme_count_from_source(text: &str, replaced_range: UiTextRange) -> (usize, usize) {
    let start = clamp_grapheme_boundary(text, replaced_range.start);
    let end = clamp_grapheme_boundary(text, replaced_range.end).max(start);
    (
        text[..start].graphemes(true).count() + text[end..].graphemes(true).count(),
        start.saturating_add(text.len().saturating_sub(end)),
    )
}

fn usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    metadata.attributes.get(key).and_then(|value| match value {
        toml::Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        toml::Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    })
}

fn positive_usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    usize_attribute(metadata, key).filter(|value| *value > 0)
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
