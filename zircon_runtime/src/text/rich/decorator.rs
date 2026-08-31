use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::text::{InlineObjectRef, LinkRef, OpenTypeFeature, StyleOverride};

use super::RichTextParseError;
use super::bbcode::{apply_builtin_style, is_parser_reserved_tag, normalized_tag};
use super::inline_decorators::{IconTextDecorator, WidgetTextDecorator};

/// Mutable neutral output supplied to a registered BBCode decorator.
///
/// The parser initializes this value from the enclosing tag stack. A
/// decorator may refine the inherited style, attach a controlled link, or
/// emit one inline object. Rendering and interaction still flow through the
/// shared rich-text contracts; decorators do not receive UI or GPU access.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RichTextDecoration {
    /// Style inherited from enclosing tags and refined by this occurrence.
    pub style: StyleOverride,
    /// Optional inline object emitted in place of the tag occurrence.
    pub inline: Option<InlineObjectRef>,
    /// Optional controlled link metadata inherited or supplied by the decorator.
    pub link: Option<LinkRef>,
}

/// Extends BBCode without adding syntax-specific branches to the parser.
pub trait RichTextDecorator: Send + Sync {
    /// Returns the tag name owned by this decorator.
    fn tag(&self) -> &str;

    /// Applies the tag value to the inherited neutral decoration.
    ///
    /// Returning `false` rejects this occurrence while preserving its inner
    /// text, matching the parser's fail-closed unknown-tag behavior.
    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Failure returned while extending a configurable rich-text parser.
pub enum RichTextDecoratorRegistrationError {
    /// This parser can no longer publish a unique decorator generation.
    GenerationExhausted,
    /// The normalized tag is empty or contains unsupported characters.
    InvalidTag(String),
    /// The tag belongs to a built-in/parser owner or another registration.
    DuplicateTag(String),
}

impl Display for RichTextDecoratorRegistrationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenerationExhausted => {
                write!(formatter, "rich-text decorator generation is exhausted")
            }
            Self::InvalidTag(tag) => write!(formatter, "invalid rich-text decorator tag `{tag}`"),
            Self::DuplicateTag(tag) => {
                write!(
                    formatter,
                    "rich-text decorator tag `{tag}` is already registered"
                )
            }
        }
    }
}

impl Error for RichTextDecoratorRegistrationError {}

struct BuiltinTextDecorator {
    tag: &'static str,
}

impl RichTextDecorator for BuiltinTextDecorator {
    fn tag(&self) -> &str {
        self.tag
    }

    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool {
        apply_builtin_style(self.tag, value, &mut decoration.style)
    }
}

pub(super) struct DecoratorRegistry {
    decorators: HashMap<String, Box<dyn RichTextDecorator>>,
}

impl DecoratorRegistry {
    pub(super) fn with_builtins() -> Self {
        let mut registry = Self {
            decorators: HashMap::with_capacity(11),
        };
        for tag in [
            "b", "i", "u", "s", "color", "bgcolor", "size", "font", "code",
        ] {
            registry.insert_builtin(BuiltinTextDecorator { tag });
        }
        registry.insert_builtin(IconTextDecorator);
        registry.insert_builtin(WidgetTextDecorator);
        registry
    }

    pub(super) fn register(
        &mut self,
        decorator: impl RichTextDecorator + 'static,
    ) -> Result<(), RichTextDecoratorRegistrationError> {
        let raw_tag = decorator.tag();
        let Some(tag) = normalized_tag(raw_tag) else {
            return Err(RichTextDecoratorRegistrationError::InvalidTag(
                raw_tag.to_string(),
            ));
        };
        if is_parser_reserved_tag(&tag) {
            return Err(RichTextDecoratorRegistrationError::DuplicateTag(tag));
        }
        match self.decorators.entry(tag) {
            Entry::Vacant(entry) => {
                entry.insert(Box::new(decorator));
                Ok(())
            }
            Entry::Occupied(entry) => Err(RichTextDecoratorRegistrationError::DuplicateTag(
                entry.key().clone(),
            )),
        }
    }

    pub(super) fn apply(
        &self,
        tag: &str,
        value: Option<&str>,
        decoration: &mut RichTextDecoration,
        max_decorator_metadata_bytes_per_call: usize,
    ) -> Result<bool, RichTextParseError> {
        let Some(decorator) = self.decorators.get(tag) else {
            return Ok(false);
        };
        let accepted = catch_unwind(AssertUnwindSafe(|| decorator.decorate(value, decoration)))
            .map_err(|_| RichTextParseError::DecoratorPanicked {
                tag: tag.to_string(),
            })?;
        if accepted {
            let attempted_bytes = retained_metadata_bytes(
                &decoration.style,
                decoration.inline.as_ref(),
                decoration.link.as_ref(),
            );
            if attempted_bytes > max_decorator_metadata_bytes_per_call {
                return Err(RichTextParseError::DecoratorMetadataBudgetExceeded {
                    tag: tag.to_string(),
                    attempted_bytes,
                    max_bytes: max_decorator_metadata_bytes_per_call,
                });
            }
        }
        Ok(accepted)
    }

    fn insert_builtin(&mut self, decorator: impl RichTextDecorator + 'static) {
        let tag = decorator.tag().to_string();
        match self.decorators.entry(tag) {
            Entry::Vacant(entry) => {
                entry.insert(Box::new(decorator));
            }
            Entry::Occupied(entry) => {
                panic!("duplicate built-in decorator tag `{}`", entry.key());
            }
        }
    }
}

pub(super) fn retained_metadata_bytes(
    style: &StyleOverride,
    inline: Option<&InlineObjectRef>,
    link: Option<&LinkRef>,
) -> usize {
    style
        .family
        .as_ref()
        .map_or(0, |family| family.0.len())
        .saturating_add(style.features.as_ref().map_or(0, |features| {
            features.len().saturating_mul(size_of::<OpenTypeFeature>())
        }))
        .saturating_add(link.map_or(0, |link| link.retained_heap_bytes()))
        .saturating_add(match inline {
            Some(InlineObjectRef::Image {
                alternative_text,
                tooltip,
                ..
            }) => alternative_text
                .as_ref()
                .map_or(0, String::len)
                .saturating_add(tooltip.as_ref().map_or(0, String::len)),
            Some(InlineObjectRef::Icon {
                alternative_text, ..
            }) => alternative_text.as_ref().map_or(0, String::len),
            Some(InlineObjectRef::Widget { .. }) | None => 0,
        })
}
