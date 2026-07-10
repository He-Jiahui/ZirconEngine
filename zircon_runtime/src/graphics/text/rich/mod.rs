use crate::core::framework::render::{RichParseResult, RichTextFormat};

mod bbcode;
mod decorator;
mod parser;

pub(crate) fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    parser::parse(
        markup,
        format,
        &decorator::DecoratorRegistry::with_builtins(),
    )
}

#[cfg(test)]
mod tests;
