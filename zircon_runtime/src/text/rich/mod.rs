use crate::text::{RichParseResult, RichTextFormat};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, OnceLock,
};

mod artifact_handle;
mod bbcode;
mod bbcode_blocks;
mod bbcode_table;
mod compiled;
mod decorator;
mod emoji_shortcode;
mod html_subset;
mod inline_decorators;
mod parser;

pub(crate) use artifact_handle::{
    register_compiled_rich_text_artifact, resolve_compiled_rich_text_artifact,
};
pub use compiled::CompiledRichText;
pub(crate) use compiled::RichTextParserGeneration;
pub use decorator::{RichTextDecoration, RichTextDecorator, RichTextDecoratorRegistrationError};
pub use emoji_shortcode::EmojiShortcodeRegistrationError;

/// Configurable rich-text parser with the built-in safe decorators installed.
pub struct RichTextParser {
    decorators: decorator::DecoratorRegistry,
    emoji_shortcodes: emoji_shortcode::EmojiShortcodeRegistry,
    parser_identity: u64,
    decorator_generation: u64,
    emoji_generation: u64,
}

impl Default for RichTextParser {
    fn default() -> Self {
        Self {
            decorators: decorator::DecoratorRegistry::with_builtins(),
            emoji_shortcodes: emoji_shortcode::EmojiShortcodeRegistry::with_builtins(),
            parser_identity: next_parser_identity(),
            decorator_generation: 1,
            emoji_generation: 1,
        }
    }
}

impl RichTextParser {
    /// Registers one BBCode decorator on this parser instance.
    pub fn register_decorator(
        &mut self,
        decorator: impl RichTextDecorator + 'static,
    ) -> Result<(), RichTextDecoratorRegistrationError> {
        self.decorators.register(decorator)?;
        self.decorator_generation = next_generation(self.decorator_generation);
        Ok(())
    }

    /// Registers one parser-local `:name:` replacement containing one grapheme.
    pub fn register_emoji_shortcode(
        &mut self,
        name: &str,
        replacement: &str,
    ) -> Result<(), EmojiShortcodeRegistrationError> {
        self.emoji_shortcodes.register(name, replacement)?;
        self.emoji_generation = next_generation(self.emoji_generation);
        Ok(())
    }

    /// Parses markup through the selected safe rich-text format.
    pub fn parse(&self, markup: &str, format: RichTextFormat) -> RichParseResult {
        self.compile(markup, format).parsed().clone()
    }

    /// Compiles markup once and shares the canonical artifact across consumers.
    pub fn compile(&self, markup: &str, format: RichTextFormat) -> Arc<CompiledRichText> {
        let generation = self.generation();
        crate::text::cache::cached_compiled_rich_text(markup, format, generation, |markup| {
            let parsed = parser::parse(
                markup.as_ref(),
                format,
                &self.decorators,
                &self.emoji_shortcodes,
            );
            CompiledRichText::new(markup, format, generation, parsed)
        })
    }

    const fn generation(&self) -> RichTextParserGeneration {
        RichTextParserGeneration {
            parser_identity: self.parser_identity,
            decorator_generation: self.decorator_generation,
            emoji_generation: self.emoji_generation,
        }
    }
}

pub(crate) fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    shared_builtin_parser().parse(markup, format)
}

pub(crate) fn compile_rich_text(markup: &str, format: RichTextFormat) -> Arc<CompiledRichText> {
    shared_builtin_parser().compile(markup, format)
}

pub(crate) fn lookup_compiled_rich_text(
    markup: &str,
    format: RichTextFormat,
) -> Option<Arc<CompiledRichText>> {
    let parser = shared_builtin_parser();
    crate::text::cache::lookup_cached_compiled_rich_text(markup, format, parser.generation())
}

fn shared_builtin_parser() -> &'static RichTextParser {
    static PARSER: OnceLock<RichTextParser> = OnceLock::new();
    PARSER.get_or_init(RichTextParser::default)
}

fn next_parser_identity() -> u64 {
    static NEXT_IDENTITY: AtomicU64 = AtomicU64::new(1);
    NEXT_IDENTITY.fetch_add(1, Ordering::Relaxed).max(1)
}

const fn next_generation(generation: u64) -> u64 {
    let next = generation.wrapping_add(1);
    if next == 0 { 1 } else { next }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/block.rs"]
mod block_tests;

#[cfg(test)]
#[path = "tests/table.rs"]
mod table_tests;
