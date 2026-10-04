mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::uniqueness::validate_runtime_plugin_package_option_key_uniqueness;
use super::super::projection::RuntimePluginPackageValidationProjection;

/// 使用共享清单投影检查选项键，保留原条目顺序并只对后续重复行追加诊断。
/// 调用期间不能换用另一份清单或重新排列选项。
pub(super) fn validate_duplicate_plugin_options(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, option) in package_manifest.options.iter().enumerate() {
        validate_runtime_plugin_package_option_key_uniqueness(
            option.key.as_str(),
            projection.option_key_is_duplicate(index),
            diagnostics,
        );
    }
}
