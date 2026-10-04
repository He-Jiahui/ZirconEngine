use crate::core::framework::project::ExportPackagingStrategy;
use crate::plugin::PluginFeatureBundleManifest;

// 声明列表含 LibraryEmbed 时优先选它；否则取首项，空列表仍回退到 LibraryEmbed。
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
