mod crates;
mod packaging;
mod target_modes;

use crate::core::framework::project::ProjectPluginFeatureSelection;
use crate::plugin::PluginFeatureBundleManifest;

use self::crates::assign_feature_project_selection_crates;
use self::packaging::feature_project_selection_packaging;
use self::target_modes::feature_project_selection_target_modes;

// 从特性清单归并目标、打包方式和模块 crate 默认值；项目显式选择随后仍可覆盖默认字段。
pub(in crate::plugin::runtime_plugin) fn project_selection_from_feature_manifest(
    feature: &PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    let mut selection = ProjectPluginFeatureSelection::new(feature.id.clone())
        .enabled(feature.enabled_by_default)
        .with_packaging(feature_project_selection_packaging(feature))
        .with_target_modes(feature_project_selection_target_modes(feature));
    assign_feature_project_selection_crates(feature, selection)
}
