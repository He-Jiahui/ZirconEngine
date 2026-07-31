use std::borrow::Cow;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use unicode_segmentation::UnicodeSegmentation;

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
    replacements: BTreeMap<String, String>,
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

    pub(super) fn expand<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if !text.contains(':') {
            return Cow::Borrowed(text);
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
            output.push_str(&text[copied..open]);
            output.push_str(replacement);
            copied = close + 1;
            scan = copied;
        }
        let Some(mut expanded) = expanded else {
            return Cow::Borrowed(text);
        };
        expanded.push_str(&text[copied..]);
        Cow::Owned(expanded)
    }
}

fn normalized_name(name: &str) -> Option<String> {
    let name = name.trim().to_ascii_lowercase();
    (!name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_'))
    .then_some(name)
}
