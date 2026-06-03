use crate::plugin::{PluginFeatureBundleManifest, PluginModuleKind, ProjectPluginFeatureSelection};

pub(super) fn assign_feature_project_selection_crates(
    feature: &PluginFeatureBundleManifest,
    mut selection: ProjectPluginFeatureSelection,
) -> ProjectPluginFeatureSelection {
    if let Some(crate_name) = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .map(|module| module.crate_name.clone())
    {
        selection = selection.with_runtime_crate(crate_name);
    }
    with_optional_editor_crate(selection, feature_editor_crate(feature))
}

fn feature_editor_crate(feature: &PluginFeatureBundleManifest) -> Option<String> {
    feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Editor)
        .map(|module| module.crate_name.clone())
}

fn with_optional_editor_crate(
    selection: ProjectPluginFeatureSelection,
    crate_name: Option<String>,
) -> ProjectPluginFeatureSelection {
    match crate_name {
        Some(crate_name) => selection.with_editor_crate(crate_name),
        None => selection,
    }
}
