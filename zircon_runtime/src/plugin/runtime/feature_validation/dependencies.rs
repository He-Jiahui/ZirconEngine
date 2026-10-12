#[path = "dependencies/owner.rs"]
mod owner;
#[path = "dependencies/pairs.rs"]
mod pairs;
#[path = "dependencies/presence.rs"]
mod presence;
#[path = "../dependencies/primary_count.rs"]
mod primary_count;
#[path = "dependencies/row.rs"]
mod row;
#[path = "dependencies/rows.rs"]
mod rows;

use super::projection::RuntimePluginFeatureValidationProjection;
use crate::plugin::PluginFeatureBundleManifest;

/// 功能必须依赖一个归属插件，并可声明额外的插件能力依赖。
/// 此处审查清单约束；实际插件启用状态与能力是否可用由目录解析阶段判断。
pub(super) fn validate_runtime_plugin_feature_dependencies(
    feature: &PluginFeatureBundleManifest,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    presence::validate_runtime_plugin_feature_dependency_presence(feature, diagnostics);
    rows::validate_runtime_plugin_feature_dependency_rows(feature, projection, diagnostics);
}
