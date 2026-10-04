mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::uniqueness::validate_runtime_plugin_package_component_type_uniqueness;
use super::super::projection::RuntimePluginPackageValidationProjection;

/// 校验包内组件类型身份；投影必须来自同一清单，且构建后组件行序不能改变。
/// 与 UI 组件分属独立注册域，同名不会在这里交叉判重。
pub(super) fn validate_duplicate_components(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, component) in package_manifest.components.iter().enumerate() {
        validate_runtime_plugin_package_component_type_uniqueness(
            component.type_id.as_str(),
            projection.component_type_id_is_duplicate(index),
            diagnostics,
        );
    }
}
