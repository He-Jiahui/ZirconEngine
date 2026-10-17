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
/// 目录拥有传入的声明；查询引用受目录借用约束，克隆后的目录可独立修改。
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
    /// 接管声明，并在结构校验通过后新增或替换同 ID 项。
    ///
    /// # 参数
    /// - `descriptor`：调用方交付的完整声明；方法取得其所有权。
    ///
    /// # 返回与错误
    /// `Ok(true)` 表示已写入变化，`Ok(false)` 表示与已有声明完全相同。
    /// 校验错误直接返回，原目录及修订号保持不变。
    ///
    /// # 参考
    /// [`Self::try_register`] 使用同一实现；[`validate_component_descriptor`] 定义准入检查。
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
    /// 从零开始饱和增加；达到 `u64::MAX` 后，后续变更仍写入目录但不再改变此值。
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the descriptor for a component id.
    /// `component_id` 按原字符串精确查找；未知 ID 返回 `None`，引用借用当前目录。
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
    /// 顺序由字符串键决定；迭代项借用目录，不能跨越对目录的可变借用继续使用。
    pub fn component_ids(&self) -> impl Iterator<Item = &str> {
        self.descriptors.keys().map(String::as_str)
    }

    /// Iterates component categories represented by the registry.
    /// 每类至多出现一次，遵循固定类别次序；结果拥有类别值，不借用描述符。
    pub fn categories(&self) -> UiComponentCategoryIter {
        unique_component_categories(
            self.descriptors
                .values()
                .map(|descriptor| descriptor.category),
        )
    }

    /// Iterates all registered descriptors in deterministic component-id order.
    /// 迭代项借用目录；消费方可按需克隆描述符，形成独立快照。
    pub fn descriptors(&self) -> impl Iterator<Item = &UiComponentDescriptor> {
        self.descriptors.values()
    }

    /// Iterates registered descriptors that belong to a component category.
    /// 过滤后仍保留组件 ID 次序；结果引用的有效期受目录借用约束。
    pub fn descriptors_in_category(
        &self,
        category: UiComponentCategory,
    ) -> impl Iterator<Item = &UiComponentDescriptor> {
        self.descriptors
            .values()
            .filter(move |descriptor| descriptor.category == category)
    }

    /// 按宿主能力选择可用声明；返回值借用当前目录。渲染能力还须由后续渲染或编译消费方检查。
    ///
    /// # 参数与返回
    /// `host_capabilities` 是本次查询使用的能力集合；返回集合由调用方持有，条目仍借用目录。
    /// 结果保持组件 ID 次序，不匹配的项直接省略；空结果不表示目录本身为空。
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
    /// 返回快照独立拥有字符串与模板，之后修改目录不会回写已有条目。
    /// 排序依次比较类别、排序键、展示名与组件 ID；此入口不检查实际渲染端支持。
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
