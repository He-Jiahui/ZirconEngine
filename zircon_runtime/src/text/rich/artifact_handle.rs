use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiRichTextArtifactHandle;

use super::CompiledRichText;

#[derive(Clone, Debug)]
struct CompiledRichTextArtifactIdentity(Arc<CompiledRichText>);

impl PartialEq for CompiledRichTextArtifactIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

impl Eq for CompiledRichTextArtifactIdentity {}

pub(crate) fn register_compiled_rich_text_artifact(
    rich: Arc<CompiledRichText>,
) -> UiRichTextArtifactHandle {
    let identity = CompiledRichTextArtifactIdentity(Arc::clone(&rich));
    UiRichTextArtifactHandle::from_runtime_artifact_with_identity(rich, identity)
}

pub(crate) fn resolve_compiled_rich_text_artifact(
    handle: &UiRichTextArtifactHandle,
) -> Option<Arc<CompiledRichText>> {
    handle.downcast_runtime_artifact().or_else(|| {
        crate::text::runtime_artifact::resolve_compiled_rich_text_from_composite(handle)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{RichTextFormat, RichTextParser};

    #[test]
    fn compiled_rich_artifact_identity_tracks_source_format_and_parser_generation() {
        let parser = RichTextParser::default();
        let first = parser
            .compile("[url=first]same[/url]", RichTextFormat::BbCodeV1)
            .expect("test rich source fits parser budgets");
        let same = parser
            .compile("[url=first]same[/url]", RichTextFormat::BbCodeV1)
            .expect("test rich source fits parser budgets");
        let different_source = parser
            .compile("[url=second]same[/url]", RichTextFormat::BbCodeV1)
            .expect("test rich source fits parser budgets");
        let different_format = parser
            .compile("[url=first]same[/url]", RichTextFormat::Plain)
            .expect("test rich source fits parser budgets");
        let different_generation = RichTextParser::default()
            .compile("[url=first]same[/url]", RichTextFormat::BbCodeV1)
            .expect("test rich source fits parser budgets");
        let first = register_compiled_rich_text_artifact(first);
        let same = register_compiled_rich_text_artifact(same);
        let different_source = register_compiled_rich_text_artifact(different_source);
        let different_format = register_compiled_rich_text_artifact(different_format);
        let different_generation = register_compiled_rich_text_artifact(different_generation);

        assert_eq!(first, same);
        assert_ne!(first, different_source);
        assert_ne!(first, different_format);
        assert_ne!(first, different_generation);
    }
}
