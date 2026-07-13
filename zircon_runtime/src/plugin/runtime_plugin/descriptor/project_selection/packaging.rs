use crate::core::framework::project::ExportPackagingStrategy;

use super::super::RuntimePluginDescriptor;

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
