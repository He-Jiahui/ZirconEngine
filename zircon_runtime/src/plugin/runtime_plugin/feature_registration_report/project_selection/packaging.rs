use crate::core::framework::project::ExportPackagingStrategy;
use crate::plugin::PluginFeatureBundleManifest;

pub(super) fn feature_project_selection_packaging(
    feature: &PluginFeatureBundleManifest,
) -> ExportPackagingStrategy {
    if feature
        .default_packaging
        .contains(&ExportPackagingStrategy::LibraryEmbed)
    {
        return ExportPackagingStrategy::LibraryEmbed;
    }
    feature
        .default_packaging
        .first()
        .copied()
        .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
}
