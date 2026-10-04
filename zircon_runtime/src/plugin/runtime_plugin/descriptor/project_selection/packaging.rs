use crate::core::framework::project::ExportPackagingStrategy;

use super::super::RuntimePluginDescriptor;

// 项目默认选项偏向库嵌入；无可用策略时保留可投影的默认值，清单校验另行报告缺失。
pub(super) fn descriptor_project_selection_packaging(
    descriptor: &RuntimePluginDescriptor,
) -> ExportPackagingStrategy {
    if descriptor
        .default_packaging
        .contains(&ExportPackagingStrategy::LibraryEmbed)
    {
        return ExportPackagingStrategy::LibraryEmbed;
    }
    descriptor
        .default_packaging
        .first()
        .copied()
        .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
}
