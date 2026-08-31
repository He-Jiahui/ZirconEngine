mod admission;
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
/// Crate-internal parse/cache bridge; public callers use `RichTextParser`.
pub(crate) mod parser_registry;
mod resource_admission;

pub use admission::{RichParseBudget, RichTextContentTrust, RichTextParseError};
pub(crate) use artifact_handle::{
    register_compiled_rich_text_artifact, resolve_compiled_rich_text_artifact,
};
pub(crate) use compiled::RichTextParserGeneration;
pub use compiled::{CompiledRichText, RichTextDependency};
pub use decorator::{RichTextDecoration, RichTextDecorator, RichTextDecoratorRegistrationError};
pub use emoji_shortcode::EmojiShortcodeRegistrationError;
pub use parser_registry::RichTextParser;

pub(super) const INLINE_OBJECT_REPLACEMENT: &str = "\u{fffc}";

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/inline_semantics.rs"]
mod inline_semantics_tests;

#[cfg(test)]
#[path = "tests/block.rs"]
mod block_tests;

#[cfg(test)]
#[path = "tests/table.rs"]
mod table_tests;

#[cfg(test)]
#[path = "tests/admission.rs"]
mod admission_tests;

#[cfg(test)]
#[path = "tests/bidi_security.rs"]
mod bidi_security_tests;
