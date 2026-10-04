use std::collections::BTreeSet;

use zircon_runtime_interface::ui::tree::UiTree;

use super::resolve::resolve_style;
use crate::text::font::DEFAULT_UI_FONT_ASSET;

/// 为 Runtime 会话的字体准入/占用阶段枚举保留树依赖，包含尚未绘制的节点并去重排序。
/// 始终保留默认字体依赖，供缺少显式字体的文字回退；返回引用而不在绘制模块内加载项目资源。
pub(in crate::ui::surface) fn text_font_asset_dependencies(tree: &UiTree) -> Vec<String> {
    let mut dependencies = BTreeSet::from([DEFAULT_UI_FONT_ASSET.to_string()]);
    for node in tree.nodes.values() {
        let style = resolve_style(node.template_metadata.as_ref());
        if let Some(font) = style.font.filter(|font| !font.trim().is_empty()) {
            dependencies.insert(font);
        }
    }
    dependencies.into_iter().collect()
}

#[cfg(test)]
#[path = "tests/font_dependencies.rs"]
mod tests;
