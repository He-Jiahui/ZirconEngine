use super::CompiledRichText;
use std::sync::Arc;
use zircon_runtime_interface::ui::surface::UiRichTextArtifactHandle;

pub(crate) fn register_compiled_rich_text_artifact(
    rich: Arc<CompiledRichText>,
) -> UiRichTextArtifactHandle {
    UiRichTextArtifactHandle::from_runtime_artifact(rich)
}

pub(crate) fn resolve_compiled_rich_text_artifact(
    handle: &UiRichTextArtifactHandle,
) -> Option<Arc<CompiledRichText>> {
    handle.downcast_runtime_artifact()
}
