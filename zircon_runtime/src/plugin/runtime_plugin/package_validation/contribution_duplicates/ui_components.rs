mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::uniqueness::validate_runtime_plugin_package_ui_component_id_uniqueness;
use super::super::projection::RuntimePluginPackageValidationProjection;

/// 校验包内 UI 组件身份；投影中的行号必须对应当前清单，而非运行时注册表的顺序。
/// 此约束属于包元数据，即使当前构建不启用 UI 注册也会检查。
pub(super) fn validate_duplicate_ui_components(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, component) in package_manifest.ui_components.iter().enumerate() {
        validate_runtime_plugin_package_ui_component_id_uniqueness(
            component.component_id.as_str(),
            projection.ui_component_id_is_duplicate(index),
            diagnostics,
        );
    }
}
