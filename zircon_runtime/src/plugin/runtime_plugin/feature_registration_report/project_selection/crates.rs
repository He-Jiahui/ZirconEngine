use crate::core::framework::project::ProjectPluginFeatureSelection;
use crate::plugin::{PluginFeatureBundleManifest, PluginModuleKind};

pub(super) fn assign_feature_project_selection_crates(
    feature: &PluginFeatureBundleManifest,
    mut selection: ProjectPluginFeatureSelection,
) -> ProjectPluginFeatureSelection {
    let (runtime_crate, editor_crate) = feature_module_crates(feature);
    if let Some(crate_name) = runtime_crate {
        selection = selection.with_runtime_crate(crate_name);
    }
    with_optional_editor_crate(selection, editor_crate)
}

fn feature_module_crates(
    feature: &PluginFeatureBundleManifest,
) -> (Option<String>, Option<String>) {
    let mut runtime_crate = None;
    let mut editor_crate = None;
    for module in &feature.modules {
        match module.kind {
            PluginModuleKind::Runtime if runtime_crate.is_none() => {
                runtime_crate = Some(module.crate_name.clone());
            }
            PluginModuleKind::Editor if editor_crate.is_none() => {
                editor_crate = Some(module.crate_name.clone());
            }
            _ => {}
        }
        if runtime_crate.is_some() && editor_crate.is_some() {
            break;
        }
    }
    (runtime_crate, editor_crate)
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

#[cfg(test)]
#[path = "crates/single_pass_module_crate_tests.rs"]
mod single_pass_module_crate_tests;
