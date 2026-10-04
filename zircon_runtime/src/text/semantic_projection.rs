//! 从已注册的编译富文本工件读取可访问性文本，避免重新解析源标记。

use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiRichTextArtifactHandle;

use super::{resolve_compiled_rich_text_artifact, CompiledRichText, RichTextFormat};

/// Generation-owned semantic view of one compiled rich-text source.
///
/// Retaining the compiled artifact keeps accessibility on the same parser
/// generation as layout and paint. The projection never reparses source markup.
pub(crate) struct RichSemanticProjection {
    compiled: Arc<CompiledRichText>,
}

impl RichSemanticProjection {
    pub(crate) fn visible_text(&self) -> &str {
        self.compiled.semantic_text()
    }

    pub(crate) fn shares_source_generation(&self, other: &Self) -> bool {
        self.compiled.generation() == other.compiled.generation()
    }
}

pub(crate) fn resolve_rich_semantic_projection(
    handle: &UiRichTextArtifactHandle,
    source_markup: &str,
    format: RichTextFormat,
) -> Option<RichSemanticProjection> {
    let compiled = resolve_compiled_rich_text_artifact(handle)?;
    from_compiled_rich_semantic_projection(compiled, source_markup, format)
}

pub(crate) fn from_compiled_rich_semantic_projection(
    compiled: Arc<CompiledRichText>,
    source_markup: &str,
    format: RichTextFormat,
) -> Option<RichSemanticProjection> {
    (compiled.source_markup() == source_markup && compiled.format() == format)
        .then_some(RichSemanticProjection { compiled })
}

#[cfg(test)]
#[path = "tests/semantic_projection.rs"]
mod tests;
