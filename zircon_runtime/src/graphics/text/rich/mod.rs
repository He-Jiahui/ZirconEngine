use crate::core::framework::render::{RichParseResult, RichTextFormat};

mod bbcode;
mod bbcode_blocks;
mod bbcode_table;
mod decorator;
mod emoji_shortcode;
mod html_subset;
mod inline_decorators;
mod parser;

pub use decorator::{RichTextDecoration, RichTextDecorator, RichTextDecoratorRegistrationError};
pub use emoji_shortcode::EmojiShortcodeRegistrationError;

/// Configurable rich-text parser with the built-in safe decorators installed.
pub struct RichTextParser {
    decorators: decorator::DecoratorRegistry,
    emoji_shortcodes: emoji_shortcode::EmojiShortcodeRegistry,
}

impl Default for RichTextParser {
    fn default() -> Self {
        Self {
            decorators: decorator::DecoratorRegistry::with_builtins(),
            emoji_shortcodes: emoji_shortcode::EmojiShortcodeRegistry::with_builtins(),
        }
    }
}

impl RichTextParser {
    /// Registers one BBCode decorator on this parser instance.
    pub fn register_decorator(
        &mut self,
        decorator: impl RichTextDecorator + 'static,
    ) -> Result<(), RichTextDecoratorRegistrationError> {
        self.decorators.register(decorator)
    }

    /// Registers one parser-local `:name:` replacement containing one grapheme.
    pub fn register_emoji_shortcode(
        &mut self,
        name: &str,
        replacement: &str,
    ) -> Result<(), EmojiShortcodeRegistrationError> {
        self.emoji_shortcodes.register(name, replacement)
    }

    /// Parses markup through the selected safe rich-text format.
    pub fn parse(&self, markup: &str, format: RichTextFormat) -> RichParseResult {
        parser::parse(markup, format, &self.decorators, &self.emoji_shortcodes)
    }
}

pub(crate) fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    RichTextParser::default().parse(markup, format)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/block.rs"]
mod block_tests;

#[cfg(test)]
#[path = "tests/table.rs"]
mod table_tests;
