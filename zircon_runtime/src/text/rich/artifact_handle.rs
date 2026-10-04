use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiRichTextArtifactHandle;

use super::CompiledRichText;

/// UI 句柄的语义身份包含完整编译结果；相同可见字串仍可能有不同链接、格式或解析器代际。
#[derive(Clone, Debug)]
struct CompiledRichTextArtifactIdentity(Arc<CompiledRichText>);

impl PartialEq for CompiledRichTextArtifactIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

impl Eq for CompiledRichTextArtifactIdentity {}

/// UI 布局发布编译产物时建立可比较的运行时句柄，供绘制、链接命中和无障碍读取同一语义快照。
pub(crate) fn register_compiled_rich_text_artifact(
    rich: Arc<CompiledRichText>,
) -> UiRichTextArtifactHandle {
    let identity = CompiledRichTextArtifactIdentity(Arc::clone(&rich));
    UiRichTextArtifactHandle::from_runtime_artifact_with_identity(rich, identity)
}

/// 消费端从纯富文本或复合文本句柄取回编译结果；句柄仍须指向存活的运行时产物。
pub(crate) fn resolve_compiled_rich_text_artifact(
    handle: &UiRichTextArtifactHandle,
) -> Option<Arc<CompiledRichText>> {
    handle.downcast_runtime_artifact().or_else(|| {
        crate::text::runtime_artifact::resolve_compiled_rich_text_from_composite(handle)
    })
}

#[cfg(test)]
#[path = "tests/artifact_handle.rs"]
mod tests;
