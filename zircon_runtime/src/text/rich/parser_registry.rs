use std::fmt::{Debug, Formatter};
use std::num::NonZeroU64;
#[cfg(test)]
use std::sync::OnceLock;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
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
mod tests {
    use std::num::NonZeroU64;
    use std::sync::{Arc, atomic::AtomicU64};

    use crate::text::{RichTextDecoration, RichTextDecorator, RichTextFormat};

    use super::{
        DecoratorRegistry, EmojiShortcodeRegistrationError, EmojiShortcodeRegistry,
        RichParseBudget, RichTextDecoratorRegistrationError, RichTextParseError, RichTextParser,
        next_generation, take_next_parser_identity,
    };

    struct GenerationDecorator;

    impl RichTextDecorator for GenerationDecorator {
        fn tag(&self) -> &str {
            "generation-test"
        }

        fn decorate(&self, _value: Option<&str>, _decoration: &mut RichTextDecoration) -> bool {
            true
        }
    }

    #[test]
    fn parser_identity_and_generation_exhaustion_never_reuse_cache_identity() {
        let local_identity = AtomicU64::new(u64::MAX - 1);
        assert_eq!(
            take_next_parser_identity(&local_identity).map(|identity| identity.get()),
            Some(u64::MAX - 1)
        );
        assert_eq!(take_next_parser_identity(&local_identity), None);
        assert_eq!(take_next_parser_identity(&local_identity), None);
        assert_eq!(next_generation(u64::MAX - 1), Some(u64::MAX));
        assert_eq!(next_generation(u64::MAX), None);

        let mut parser = RichTextParser {
            decorators: DecoratorRegistry::with_builtins(),
            emoji_shortcodes: EmojiShortcodeRegistry::with_builtins(),
            parser_identity: NonZeroU64::new(1),
            decorator_generation: u64::MAX,
            emoji_generation: u64::MAX,
            budget: RichParseBudget::default(),
            cache: CompiledRichTextCacheOwner::default(),
        };
        assert_eq!(
            parser.register_decorator(GenerationDecorator),
            Err(RichTextDecoratorRegistrationError::GenerationExhausted)
        );
        let mut decoration = RichTextDecoration::default();
        assert_eq!(
            parser
                .decorators
                .apply("generation-test", None, &mut decoration, usize::MAX,),
            Ok(false)
        );
        assert_eq!(
            parser.register_emoji_shortcode("generation_test", "x"),
            Err(EmojiShortcodeRegistrationError::GenerationExhausted)
        );
        assert_eq!(
            parser
                .emoji_shortcodes
                .expand(":generation_test:", 0, usize::MAX)
                .expect("unregistered shortcode remains literal"),
            ":generation_test:"
        );

        parser.parser_identity = None;
        assert!(matches!(
            parser.compile("plain", RichTextFormat::Plain),
            Err(RichTextParseError::ParserIdentityExhausted)
        ));
    }

    #[test]
    fn provider_generation_publication_retires_cache_without_revoking_last_use_artifacts() {
        let mut parser = RichTextParser::default();
        let source = "[generation-test]x[/generation-test] :zircon:";
        let before_registration = parser
            .compile(source, RichTextFormat::BbCodeV1)
            .expect("baseline artifact compiles");
        assert_eq!(parser.compiled_cache_report().resident_entries, 1);

        parser
            .register_decorator(GenerationDecorator)
            .expect("decorator registration advances the parser generation");
        assert_eq!(parser.compiled_cache_report().resident_entries, 0);
        assert_eq!(before_registration.source_markup(), source);

        let after_decorator = parser
            .compile(source, RichTextFormat::BbCodeV1)
            .expect("new decorator generation compiles");
        assert!(!Arc::ptr_eq(&before_registration, &after_decorator));
        assert_eq!(parser.compiled_cache_report().resident_entries, 1);
        assert!(matches!(
            parser.register_decorator(GenerationDecorator),
            Err(RichTextDecoratorRegistrationError::DuplicateTag(_))
        ));
        assert_eq!(parser.compiled_cache_report().resident_entries, 1);
        assert!(Arc::ptr_eq(
            &after_decorator,
            &parser
                .lookup_compiled(source, RichTextFormat::BbCodeV1)
                .expect("failed registration preserves current-generation residency")
        ));

        parser
            .register_emoji_shortcode("zircon", "x")
            .expect("emoji registration advances the parser generation");
        assert_eq!(parser.compiled_cache_report().resident_entries, 0);
        assert_eq!(after_decorator.source_markup(), source);
    }
}
