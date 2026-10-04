//! 模板编译和编辑器查询共享的组件声明目录；它保存描述符及其修订身份，实例交互状态由各节点的状态模型持有。

use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiHostCapability, UiHostCapabilitySet,
};

use super::super::descriptor::{validate_component_descriptor, UiComponentDescriptorError};

use super::palette_view::UiComponentPaletteEntry;

const COMPONENT_CATEGORIES: [UiComponentCategory; 8] = [
    UiComponentCategory::Visual,
    UiComponentCategory::Input,
    UiComponentCategory::Numeric,
    UiComponentCategory::Selection,
    UiComponentCategory::Reference,
    UiComponentCategory::Collection,
    UiComponentCategory::Container,
    UiComponentCategory::Feedback,
];

type UiComponentCategoryIter =
    std::iter::Flatten<std::array::IntoIter<Option<UiComponentCategory>, 8>>;

/// 按稳定组件 ID 保存可编写契约。更换声明时返回变更标记并推进修订号，供编译调用链识别目录变化。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiComponentDescriptorRegistry {
    descriptors: BTreeMap<String, UiComponentDescriptor>,
    revision: u64,
}

impl UiComponentDescriptorRegistry {
    /// Creates an empty component descriptor registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers or replaces a descriptor by component id.
    pub fn register(
        &mut self,
        descriptor: UiComponentDescriptor,
    ) -> Result<bool, UiComponentDescriptorError> {
        self.try_register(descriptor)
    }

    /// 与 register 使用同一校验入口；完全相同的声明是幂等操作，校验失败时保留已有目录及修订号。
    pub fn try_register(
        &mut self,
        descriptor: UiComponentDescriptor,
    ) -> Result<bool, UiComponentDescriptorError> {
        validate_component_descriptor(&descriptor)?;
        if self.descriptors.get(&descriptor.id) == Some(&descriptor) {
            return Ok(false);
        }
        self.descriptors.insert(descriptor.id.clone(), descriptor);
        self.revision = self.revision.saturating_add(1);
        Ok(true)
    }

    /// Returns the monotonic registry revision for descriptor-set changes.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the descriptor for a component id.
    pub fn descriptor(&self, component_id: &str) -> Option<&UiComponentDescriptor> {
        self.descriptors.get(component_id)
    }

    /// Returns whether the registry has a descriptor for a component id.
    pub fn contains(&self, component_id: &str) -> bool {
        self.descriptors.contains_key(component_id)
    }

    /// Returns the number of registered component descriptors.
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// Returns whether the registry has no registered component descriptors.
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    /// Iterates registered component ids in deterministic order.
    pub fn component_ids(&self) -> impl Iterator<Item = &str> {
        self.descriptors.keys().map(String::as_str)
    }

    /// Iterates component categories represented by the registry.
    pub fn categories(&self) -> UiComponentCategoryIter {
        unique_component_categories(
            self.descriptors
                .values()
                .map(|descriptor| descriptor.category),
        )
    }

    /// Iterates all registered descriptors in deterministic component-id order.
    pub fn descriptors(&self) -> impl Iterator<Item = &UiComponentDescriptor> {
        self.descriptors.values()
    }

    /// Iterates registered descriptors that belong to a component category.
    pub fn descriptors_in_category(
        &self,
        category: UiComponentCategory,
    ) -> impl Iterator<Item = &UiComponentDescriptor> {
        self.descriptors
            .values()
            .filter(move |descriptor| descriptor.category == category)
    }

    /// 按宿主能力选择可用声明；返回值借用当前目录。渲染能力还须由后续渲染或编译消费方检查。
    pub fn descriptors_for_host(
        &self,
        host_capabilities: &UiHostCapabilitySet,
    ) -> Vec<&UiComponentDescriptor> {
        let mut descriptors = Vec::with_capacity(self.descriptors.len());
        descriptors.extend(self.descriptors.values().filter(|descriptor| {
            host_capabilities.contains_all(&descriptor.required_host_capabilities)
        }));
        descriptors
    }

    /// 给编写器生成拥有字段的调色板投影；仅声明调色板元数据且满足宿主能力的组件会出现。
    pub fn palette_entries_for_host(
        &self,
        host_capabilities: &UiHostCapabilitySet,
    ) -> Vec<UiComponentPaletteEntry> {
        super::palette_view::palette_entries_for_host(self, host_capabilities)
    }

    /// 缺少组件时返回 None，已知且能力足够时返回空集合，调用方可据此区分未知 ID 与宿主准入失败。
    pub fn missing_capabilities(
        &self,
        component_id: &str,
        host_capabilities: &UiHostCapabilitySet,
    ) -> Option<BTreeSet<UiHostCapability>> {
        self.descriptor(component_id)
            .map(|descriptor| host_capabilities.missing(&descriptor.required_host_capabilities))
    }
}

// 分类目录遵循枚举契约次序，而组件目录遵循 ID 次序；这两个顺序各自用于稳定编写结果。
fn unique_component_categories(
    categories: impl IntoIterator<Item = UiComponentCategory>,
) -> UiComponentCategoryIter {
    let mut ordered = [None; COMPONENT_CATEGORIES.len()];
    for category in categories {
        ordered[component_category_index(category)] = Some(category);
    }
    ordered.into_iter().flatten()
}

const fn component_category_index(category: UiComponentCategory) -> usize {
    match category {
        UiComponentCategory::Visual => 0,
        UiComponentCategory::Input => 1,
        UiComponentCategory::Numeric => 2,
        UiComponentCategory::Selection => 3,
        UiComponentCategory::Reference => 4,
        UiComponentCategory::Collection => 5,
        UiComponentCategory::Container => 6,
        UiComponentCategory::Feedback => 7,
    }
}

#[cfg(test)]
#[path = "tests/registry.rs"]
mod tests;
