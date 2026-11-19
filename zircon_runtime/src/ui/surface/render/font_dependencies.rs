use std::collections::BTreeSet;

use zircon_runtime_interface::ui::tree::UiTree;

use super::resolve::resolve_style;
use crate::text::font::DEFAULT_UI_FONT_ASSET;

/// 为 Runtime 会话的字体准入/占用阶段枚举保留树依赖，包含尚未绘制的节点并去重排序。
/// 始终保留默认字体依赖，供缺少显式字体的文字回退；返回引用而不在绘制模块内加载项目资源。
pub(in crate::ui::surface) fn text_font_asset_dependencies(tree: &UiTree) -> Vec<String> {
    let mut dependencies = explicit_text_font_asset_dependencies(tree);
    dependencies.insert(DEFAULT_UI_FONT_ASSET.to_string());
    dependencies.into_iter().collect()
}

/// An authored URI is a project dependency even if it equals the default URI.
pub(in crate::ui::surface) fn explicit_text_font_asset_dependencies(
    tree: &UiTree,
) -> BTreeSet<String> {
    let mut dependencies = BTreeSet::new();
    for node in tree.nodes.values() {
        let style = resolve_style(node.template_metadata.as_ref());
        if let Some(font) = style.font.filter(|font| !font.trim().is_empty()) {
            dependencies.insert(font);
        }
    }
    dependencies
}

#[cfg(test)]
#[path = "tests/font_dependencies.rs"]
mod tests;
