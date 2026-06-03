use crate::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginModuleKind,
    ProjectPluginFeatureSelection,
};

pub(super) fn project_feature_selection(
    feature: &PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    let mut selection =
        ProjectPluginFeatureSelection::new(feature.id.clone()).enabled(feature.enabled_by_default);
    selection.packaging = feature_project_selection_packaging(feature);
    assign_feature_module_crates(feature, &mut selection);
    selection
}

fn feature_project_selection_packaging(
    feature: &PluginFeatureBundleManifest,
) -> ExportPackagingStrategy {
    feature
        .default_packaging
        .iter()
        .copied()
        .find(|packaging| *packaging == ExportPackagingStrategy::LibraryEmbed)
        .or_else(|| feature.default_packaging.first().copied())
        .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
}

fn assign_feature_module_crates(
    feature: &PluginFeatureBundleManifest,
    selection: &mut ProjectPluginFeatureSelection,
) {
    for module in &feature.modules {
        match module.kind {
            PluginModuleKind::Runtime if selection.runtime_crate.is_none() => {
                selection.runtime_crate = Some(module.crate_name.clone());
            }
            PluginModuleKind::Editor if selection.editor_crate.is_none() => {
                selection.editor_crate = Some(module.crate_name.clone());
            }
            _ => {}
        }
    }
}
