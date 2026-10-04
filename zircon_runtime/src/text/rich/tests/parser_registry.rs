use std::num::NonZeroU64;
use std::sync::{atomic::AtomicU64, Arc};

use crate::text::{RichTextDecoration, RichTextDecorator, RichTextFormat};

use super::{
    next_generation, take_next_parser_identity, CompiledRichTextCacheOwner, DecoratorRegistry,
    EmojiShortcodeRegistrationError, EmojiShortcodeRegistry, RichParseBudget,
    RichTextDecoratorRegistrationError, RichTextParseError, RichTextParser,
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
