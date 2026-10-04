//! 将注册表声明投影为可序列化的编写条目；调用方获得独立字段和默认模板，供目录展示与新节点创建使用。

use serde::{Deserialize, Serialize};

use super::registry::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiDefaultNodeTemplate, UiHostCapabilitySet,
};

/// 编写调色板的交换记录；default_node 携带创建新节点的模板，component_id 连接注册表中的结构契约。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentPaletteEntry {
    pub component_id: String,
    pub display_name: String,
    pub category: UiComponentCategory,
    #[serde(default)]
    pub icon: Option<String>,
    pub sort_key: String,
    pub default_node: UiDefaultNodeTemplate,
}

pub(super) fn palette_entries_for_host(
    registry: &UiComponentDescriptorRegistry,
    host_capabilities: &UiHostCapabilitySet,
) -> Vec<UiComponentPaletteEntry> {
    let mut entries = Vec::with_capacity(registry.len());
    entries.extend(
        registry
            .descriptors()
            .filter(|descriptor| {
                host_capabilities.contains_all(&descriptor.required_host_capabilities)
            })
            .filter_map(|descriptor| {
                let metadata = descriptor.palette.as_ref()?;
                Some(UiComponentPaletteEntry {
                    component_id: descriptor.id.clone(),
                    display_name: metadata.display_name.clone(),
                    category: metadata.category,
                    icon: metadata.icon.clone(),
                    sort_key: metadata.sort_key.clone(),
                    default_node: metadata.default_node.clone(),
                })
            }),
    );
    // 唯一组件 ID 是最终排序键，使相同类别和展示字段仍具有确定次序，编写目录可稳定复现。
    entries.sort_unstable_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.sort_key.cmp(&right.sort_key))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.component_id.cmp(&right.component_id))
    });
    entries
}

#[cfg(test)]
#[path = "tests/palette_view.rs"]
mod tests;
