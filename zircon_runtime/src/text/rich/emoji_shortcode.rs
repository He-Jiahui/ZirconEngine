use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use unicode_segmentation::UnicodeSegmentation;

use super::RichTextParseError;

const BUILTIN_EMOJI_SHORTCODES: &[(&str, &str)] = &[
    ("check", "✅"),
    ("fire", "🔥"),
    ("grinning", "😀"),
    ("heart", "❤️"),
    ("rocket", "🚀"),
    ("smile", "😄"),
    ("sparkles", "✨"),
    ("thumbsup", "👍"),
    ("warning", "⚠️"),
    ("x", "❌"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
/// Failure returned while extending a parser-local emoji shortcode map.
pub enum EmojiShortcodeRegistrationError {
    /// This parser can no longer publish a unique emoji generation.
    GenerationExhausted,
    /// The normalized name is empty or contains unsupported characters.
    InvalidName(String),
    /// A replacement must be exactly one Unicode grapheme.
    InvalidReplacement(String),
    /// The name already belongs to a built-in or caller registration.
    DuplicateName(String),
}

impl Display for EmojiShortcodeRegistrationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenerationExhausted => {
                write!(formatter, "rich-text emoji generation is exhausted")
            }
            Self::InvalidName(name) => write!(formatter, "invalid emoji shortcode `{name}`"),
            Self::InvalidReplacement(value) => {
                write!(
                    formatter,
                    "emoji shortcode replacement `{value}` is not one grapheme"
                )
            }
            Self::DuplicateName(name) => {
                write!(formatter, "emoji shortcode `{name}` is already registered")
            }
        }
    }
}

impl Error for EmojiShortcodeRegistrationError {}

pub(super) struct EmojiShortcodeRegistry {
    replacements: HashMap<String, String>,
}

impl EmojiShortcodeRegistry {
    pub(super) fn with_builtins() -> Self {
        Self {
            replacements: BUILTIN_EMOJI_SHORTCODES
                .iter()
                .map(|(name, replacement)| ((*name).to_string(), (*replacement).to_string()))
                .collect(),
        }
    }

    pub(super) fn register(
        &mut self,
        name: &str,
        replacement: &str,
    ) -> Result<(), EmojiShortcodeRegistrationError> {
        let Some(name) = normalized_name(name) else {
            return Err(EmojiShortcodeRegistrationError::InvalidName(
                name.to_string(),
            ));
        };
        if replacement.graphemes(true).count() != 1 {
            return Err(EmojiShortcodeRegistrationError::InvalidReplacement(
                replacement.to_string(),
            ));
        }
        if self.replacements.contains_key(&name) {
            return Err(EmojiShortcodeRegistrationError::DuplicateName(name));
        }
        self.replacements.insert(name, replacement.to_string());
        Ok(())
    }

    pub(super) fn expand<'a>(
        &self,
        text: &'a str,
        existing_output_bytes: usize,
        max_output_bytes: usize,
    ) -> Result<Cow<'a, str>, RichTextParseError> {
        if !text.contains(':') {
            return Ok(Cow::Borrowed(text));
        }

        let mut expanded = None;
        let mut scan = 0;
        let mut copied = 0;
        while let Some(open_offset) = text[scan..].find(':') {
            let open = scan + open_offset;
            let name_start = open + 1;
            let Some(close_offset) = text[name_start..].find(':') else {
                break;
            };
            let close = name_start + close_offset;
            let name = &text[name_start..close];
            let Some(replacement) = normalized_name(name)
                .as_ref()
                .and_then(|name| self.replacements.get(name))
            else {
                scan = close + 1;
                continue;
            };
            let output = expanded.get_or_insert_with(|| String::with_capacity(text.len()));
            push_expansion_chunk(
                output,
                &text[copied..open],
                existing_output_bytes,
                max_output_bytes,
            )?;
            push_expansion_chunk(output, replacement, existing_output_bytes, max_output_bytes)?;
            copied = close + 1;
            scan = copied;
        }
        let Some(mut expanded) = expanded else {
            return Ok(Cow::Borrowed(text));
        };
        push_expansion_chunk(
            &mut expanded,
            &text[copied..],
            existing_output_bytes,
            max_output_bytes,
        )?;
        Ok(Cow::Owned(expanded))
    }
}

fn push_expansion_chunk(
    output: &mut String,
    chunk: &str,
    existing_output_bytes: usize,
    max_output_bytes: usize,
) -> Result<(), RichTextParseError> {
    let attempted_bytes = existing_output_bytes
        .checked_add(output.len())
        .and_then(|bytes| bytes.checked_add(chunk.len()))
        .unwrap_or(usize::MAX);
    if attempted_bytes > max_output_bytes {
        return Err(RichTextParseError::OutputByteBudgetExceeded {
            attempted_bytes,
            max_bytes: max_output_bytes,
        });
    }
    output.push_str(chunk);
    Ok(())
}

fn normalized_name(name: &str) -> Option<String> {
    let name = name.trim().to_ascii_lowercase();
    (!name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_'))
    .then_some(name)
}

#[cfg(test)]
#[path = "emoji_shortcode/hash_index_tests.rs"]
mod hash_index_tests;
