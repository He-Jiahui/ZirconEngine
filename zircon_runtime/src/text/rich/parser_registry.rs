use std::fmt::{Debug, Formatter};
use std::num::NonZeroU64;
#[cfg(test)]
use std::sync::OnceLock;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[cfg(test)]
use crate::text::RichParseResult;
use crate::text::RichTextFormat;

use super::{
    admission::{RichParseBudget, RichTextContentTrust, RichTextParseError},
    compiled::{CompiledRichText, RichTextParserGeneration},
    decorator::{DecoratorRegistry, RichTextDecorator, RichTextDecoratorRegistrationError},
    emoji_shortcode::{EmojiShortcodeRegistrationError, EmojiShortcodeRegistry},
};
use crate::text::cache::{CompiledRichTextCacheOwner, CompiledRichTextCacheReport};

/// Configurable rich-text parser with the built-in safe decorators installed.
pub struct RichTextParser {
    decorators: DecoratorRegistry,
    emoji_shortcodes: EmojiShortcodeRegistry,
    parser_identity: Option<NonZeroU64>,
    decorator_generation: u64,
    emoji_generation: u64,
    budget: RichParseBudget,
    cache: CompiledRichTextCacheOwner,
}

impl Debug for RichTextParser {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RichTextParser")
            .field("parser_identity", &self.parser_identity)
            .field("decorator_generation", &self.decorator_generation)
            .field("emoji_generation", &self.emoji_generation)
            .field("budget", &self.budget)
            .field("cache", &self.cache)
            .finish()
    }
}

impl Default for RichTextParser {
    fn default() -> Self {
        Self {
            decorators: DecoratorRegistry::with_builtins(),
            emoji_shortcodes: EmojiShortcodeRegistry::with_builtins(),
            parser_identity: next_parser_identity(),
            decorator_generation: 1,
            emoji_generation: 1,
            budget: RichParseBudget::default(),
            cache: CompiledRichTextCacheOwner::default(),
        }
    }
}

impl RichTextParser {
    pub fn with_budget(budget: RichParseBudget) -> Self {
        Self {
            budget,
            ..Self::default()
        }
    }

    pub const fn budget(&self) -> RichParseBudget {
        self.budget
    }

    #[cfg(test)]
    pub(crate) fn parse(
        &self,
        markup: &str,
        format: RichTextFormat,
    ) -> Result<RichParseResult, RichTextParseError> {
        let compiled = self.compile(markup, format)?;
        Ok(RichParseResult::clone(compiled.parsed()))
    }

    /// Registers one BBCode decorator on this parser instance.
    pub fn register_decorator(
        &mut self,
        decorator: impl RichTextDecorator + 'static,
    ) -> Result<(), RichTextDecoratorRegistrationError> {
        let next_generation = self.next_decorator_generation()?;
        self.decorators.register(decorator)?;
        self.decorator_generation = next_generation;
        self.cache.clear();
        Ok(())
    }

    /// Registers one parser-local `:name:` replacement containing one grapheme.
    pub fn register_emoji_shortcode(
        &mut self,
        name: &str,
        replacement: &str,
    ) -> Result<(), EmojiShortcodeRegistrationError> {
        let next_generation = self.next_emoji_generation()?;
        self.emoji_shortcodes.register(name, replacement)?;
        self.emoji_generation = next_generation;
        self.cache.clear();
        Ok(())
    }

    /// Compiles markup once and shares the canonical artifact across consumers.
    pub fn compile(
        &self,
        markup: &str,
        format: RichTextFormat,
    ) -> Result<Arc<CompiledRichText>, RichTextParseError> {
        self.compile_with_content_trust(markup, format, RichTextContentTrust::Untrusted)
    }

    /// Compiles markup under an explicit authoring trust policy.
    ///
    /// `TrustedAuthoring` must only be selected for author-controlled source. It permits balanced
    /// legacy bidi embeddings and overrides that the default untrusted entry point rejects.
    pub fn compile_with_content_trust(
        &self,
        markup: &str,
        format: RichTextFormat,
        content_trust: RichTextContentTrust,
    ) -> Result<Arc<CompiledRichText>, RichTextParseError> {
        let generation = self.generation()?;
        self.budget.admit_source(markup.len())?;
        self.cache
            .compile(markup, format, content_trust, generation, |markup| {
                let parsed = super::parser::parse(
                    markup.as_ref(),
                    format,
                    &self.decorators,
                    &self.emoji_shortcodes,
                    self.budget,
                    content_trust,
                )?;
                CompiledRichText::new_with_content_trust_and_projection_budget(
                    markup,
                    format,
                    content_trust,
                    generation,
                    parsed,
                    self.budget.max_projection_indices,
                    self.budget.admitted_semantic_text_bytes(),
                )
            })
    }

    pub(crate) fn lookup_compiled(
        &self,
        markup: &str,
        format: RichTextFormat,
    ) -> Option<Arc<CompiledRichText>> {
        let generation = self.generation().ok()?;
        self.budget.admit_source(markup.len()).ok()?;
        self.cache
            .lookup(markup, format, RichTextContentTrust::Untrusted, generation)
    }

    pub(crate) fn compiled_cache_report(&self) -> CompiledRichTextCacheReport {
        self.cache.report().with_generation(self.cache_generation())
    }

    pub(crate) fn take_compiled_cache_report(&self) -> CompiledRichTextCacheReport {
        self.cache
            .take_report()
            .with_generation(self.cache_generation())
    }

    pub(crate) fn clear_compiled_cache(&self) {
        self.cache.clear();
    }

    fn next_decorator_generation(&self) -> Result<u64, RichTextDecoratorRegistrationError> {
        self.parser_identity
            .ok_or(RichTextDecoratorRegistrationError::GenerationExhausted)?;
        next_generation(self.decorator_generation)
            .ok_or(RichTextDecoratorRegistrationError::GenerationExhausted)
    }

    fn next_emoji_generation(&self) -> Result<u64, EmojiShortcodeRegistrationError> {
        self.parser_identity
            .ok_or(EmojiShortcodeRegistrationError::GenerationExhausted)?;
        next_generation(self.emoji_generation)
            .ok_or(EmojiShortcodeRegistrationError::GenerationExhausted)
    }

    fn generation(&self) -> Result<RichTextParserGeneration, RichTextParseError> {
        let parser_identity = self
            .parser_identity
            .ok_or(RichTextParseError::ParserIdentityExhausted)?;
        Ok(RichTextParserGeneration {
            parser_identity: parser_identity.get(),
            decorator_generation: self.decorator_generation,
            emoji_generation: self.emoji_generation,
        })
    }

    fn cache_generation(&self) -> RichTextParserGeneration {
        RichTextParserGeneration {
            parser_identity: self.parser_identity.map_or(0, NonZeroU64::get),
            decorator_generation: self.decorator_generation,
            emoji_generation: self.emoji_generation,
        }
    }
}

#[cfg(test)]
pub(crate) fn parse_rich_text(
    markup: &str,
    format: RichTextFormat,
) -> Result<RichParseResult, RichTextParseError> {
    shared_builtin_parser().parse(markup, format)
}

#[cfg(test)]
pub(crate) fn compile_rich_text(
    markup: &str,
    format: RichTextFormat,
) -> Result<Arc<CompiledRichText>, RichTextParseError> {
    shared_builtin_parser().compile(markup, format)
}

#[cfg(test)]
pub(crate) fn lookup_compiled_rich_text(
    markup: &str,
    format: RichTextFormat,
) -> Option<Arc<CompiledRichText>> {
    shared_builtin_parser().lookup_compiled(markup, format)
}

#[cfg(test)]
pub(super) fn shared_builtin_parser() -> &'static RichTextParser {
    static PARSER: OnceLock<RichTextParser> = OnceLock::new();
    PARSER.get_or_init(RichTextParser::default)
}

fn next_parser_identity() -> Option<NonZeroU64> {
    static NEXT_IDENTITY: AtomicU64 = AtomicU64::new(1);
    take_next_parser_identity(&NEXT_IDENTITY)
}

fn take_next_parser_identity(next_identity: &AtomicU64) -> Option<NonZeroU64> {
    let identity = next_identity
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |identity| {
            identity.checked_add(1)
        })
        .ok()?;
    NonZeroU64::new(identity)
}

const fn next_generation(generation: u64) -> Option<u64> {
    generation.checked_add(1)
}

#[cfg(test)]
#[path = "tests/parser_registry.rs"]
mod tests;
