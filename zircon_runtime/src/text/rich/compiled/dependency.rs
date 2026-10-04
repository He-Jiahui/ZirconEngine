use crate::core::resource::ResourceId;
use crate::text::{InlineObjectRef, RichIconAssetId, RichParseResult};

/// Loadable resource dependency retained by one compiled rich-text artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichTextDependency {
    ImageTexture(ResourceId),
    IconAsset(RichIconAssetId),
}

// 解析结果只把需要外部加载的图片纹理和图标资产提升为依赖；组件占位符没有资源句柄。
// 排序后去重让编译产物暴露稳定且无重复的资源清单，供缓存和渲染消费者按值使用。
pub(super) fn collect(parsed: &RichParseResult) -> Vec<RichTextDependency> {
    let mut dependencies = parsed
        .runs
        .iter()
        .filter_map(|run| match run.inline.as_ref() {
            Some(InlineObjectRef::Image { texture, .. }) => {
                Some(RichTextDependency::ImageTexture(*texture))
            }
            Some(InlineObjectRef::Icon { asset, .. }) => {
                Some(RichTextDependency::IconAsset(*asset))
            }
            Some(InlineObjectRef::Widget { .. }) | None => None,
        })
        .collect::<Vec<_>>();
    dependencies.sort_unstable();
    dependencies.dedup();
    dependencies
}

#[cfg(test)]
#[path = "tests/dependency.rs"]
mod tests;
