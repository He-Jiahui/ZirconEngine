//! 在编译缓存 miss 时比较同资产的前后输入修订，给出有序阶段与影响摘要。
//! 分类不重新编译或修改 UI；watch 驱动的热重载执行使用独立计划。

use std::collections::BTreeSet;

use zircon_runtime_interface::ui::template::{
    UiAssetChange, UiAssetDocument, UiInvalidationReport, UiInvalidationSnapshot,
    UiInvalidationStage,
};

use super::collect_invalidation_diagnostics;

/// 无状态的缓存输入分类器；快照一致性和资产身份配对由编译缓存调用方负责。
#[derive(Clone, Debug, Default)]
pub struct UiInvalidationGraph;

impl UiInvalidationGraph {
    /// 文档内容变化要求整体重建；其他修订单独变化时选择相关阶段并合并影响。
    /// 没有前态时按首次编译报告整体重建，文档仅用于本次诊断。
    pub fn classify(
        previous: Option<&UiInvalidationSnapshot>,
        next: &UiInvalidationSnapshot,
        document: &UiAssetDocument,
    ) -> UiInvalidationReport {
        let diagnostics = collect_invalidation_diagnostics(document);
        let Some(previous) = previous else {
            return UiInvalidationReport::from_stages(
                vec![UiAssetChange::Document],
                full_rebuild_stages(),
                diagnostics,
            );
        };

        let mut changes = Vec::new();
        let mut stages = BTreeSet::new();
        let full_rebuild = previous.document != next.document;
        if full_rebuild {
            changes.push(UiAssetChange::Document);
            stages.extend(full_rebuild_stages());
        }
        if previous.widget_imports != next.widget_imports
            || previous.declared_widget_imports_revision != next.declared_widget_imports_revision
        {
            changes.push(UiAssetChange::WidgetImport);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::ImportGraph,
                    UiInvalidationStage::ComponentContract,
                    UiInvalidationStage::SelectorMatch,
                    UiInvalidationStage::StyleValue,
                    UiInvalidationStage::Layout,
                    UiInvalidationStage::Render,
                ],
            );
        }
        if previous.style_imports != next.style_imports
            || previous.declared_style_imports_revision != next.declared_style_imports_revision
        {
            changes.push(UiAssetChange::StyleImport);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::ImportGraph,
                    UiInvalidationStage::SelectorMatch,
                    UiInvalidationStage::StyleValue,
                    UiInvalidationStage::Layout,
                    UiInvalidationStage::Render,
                ],
            );
        }
        if previous.descriptor_registry_revision != next.descriptor_registry_revision {
            changes.push(UiAssetChange::DescriptorRegistry);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::DescriptorRegistry,
                    UiInvalidationStage::Layout,
                    UiInvalidationStage::Render,
                ],
            );
        }
        if previous.component_contract_revision != next.component_contract_revision {
            changes.push(UiAssetChange::ComponentContract);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::ComponentContract,
                    UiInvalidationStage::SelectorMatch,
                    UiInvalidationStage::StyleValue,
                    UiInvalidationStage::Layout,
                    UiInvalidationStage::Render,
                ],
            );
        }
        if previous.resource_dependencies_revision != next.resource_dependencies_revision {
            changes.push(UiAssetChange::ResourceDependency);
            extend_incremental_stages(
                &mut stages,
                full_rebuild,
                [
                    UiInvalidationStage::ResourceDependency,
                    UiInvalidationStage::Render,
                    UiInvalidationStage::Projection,
                ],
            );
        }
        UiInvalidationReport::from_stages(changes, stages, diagnostics)
    }
}

// 整体重建已覆盖 UI 工作影响，仍记录各修订变化，但不重复添加增量分类阶段。
fn extend_incremental_stages(
    stages: &mut BTreeSet<UiInvalidationStage>,
    full_rebuild: bool,
    additions: impl IntoIterator<Item = UiInvalidationStage>,
) {
    if !full_rebuild {
        stages.extend(additions);
    }
}

// 这里是整体重建的代表阶段集；来源解析和文档形状已经让影响转换标记全部脏域。
fn full_rebuild_stages() -> BTreeSet<UiInvalidationStage> {
    [
        UiInvalidationStage::SourceParse,
        UiInvalidationStage::DocumentShape,
        UiInvalidationStage::SelectorMatch,
        UiInvalidationStage::StyleValue,
        UiInvalidationStage::Layout,
        UiInvalidationStage::Render,
        UiInvalidationStage::Interaction,
        UiInvalidationStage::Projection,
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
#[path = "graph/tests/full_rebuild_tests.rs"]
mod full_rebuild_tests;
