use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::core::framework::render::{InlineObjectRef, LinkRef, StyleOverride};

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
    /// The normalized tag is empty or contains unsupported characters.
    InvalidTag(String),
    /// The tag belongs to a built-in/parser owner or another registration.
    DuplicateTag(String),
}

impl Display for RichTextDecoratorRegistrationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
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

struct RegisteredDecorator {
    tag: String,
    decorator: Box<dyn RichTextDecorator>,
}

pub(super) struct DecoratorRegistry {
    decorators: Vec<RegisteredDecorator>,
}

impl DecoratorRegistry {
    pub(super) fn with_builtins() -> Self {
        let mut registry = Self {
            decorators: Vec::new(),
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
        if is_parser_reserved_tag(&tag)
            || self
                .decorators
                .iter()
                .any(|registered| registered.tag == tag)
        {
            return Err(RichTextDecoratorRegistrationError::DuplicateTag(tag));
        }
        self.decorators.push(RegisteredDecorator {
            tag,
            decorator: Box::new(decorator),
        });
        Ok(())
    }

    pub(super) fn apply(
        &self,
        tag: &str,
        value: Option<&str>,
        decoration: &mut RichTextDecoration,
    ) -> bool {
        self.decorators
            .iter()
            .find(|registered| registered.tag == tag)
            .is_some_and(|registered| registered.decorator.decorate(value, decoration))
    }

    fn insert_builtin(&mut self, decorator: impl RichTextDecorator + 'static) {
        let tag = decorator.tag().to_string();
        debug_assert!(self
            .decorators
            .iter()
            .all(|registered| registered.tag != tag));
        self.decorators.push(RegisteredDecorator {
            tag,
            decorator: Box::new(decorator),
        });
    }
}
