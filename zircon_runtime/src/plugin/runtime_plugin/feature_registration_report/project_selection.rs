mod crates;
mod packaging;
mod target_modes;

use crate::plugin::{PluginFeatureBundleManifest, ProjectPluginFeatureSelection};

use self::crates::assign_feature_project_selection_crates;
use self::packaging::feature_project_selection_packaging;
use self::target_modes::feature_project_selection_target_modes;

pub(in crate::plugin::runtime_plugin) fn project_selection_from_feature_manifest(
    feature: &PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    let mut selection = ProjectPluginFeatureSelection::new(feature.id.clone())
        .enabled(feature.enabled_by_default)
        .with_packaging(feature_project_selection_packaging(feature))
        .with_target_modes(feature_project_selection_target_modes(feature));
    assign_feature_project_selection_crates(feature, selection)
}
