mod completeness;
mod fields;

use crate::plugin::PluginPackageManifest;

use self::{
    completeness::validate_runtime_plugin_package_coordinate_completeness,
    fields::RuntimePluginPackageCoordinateFields,
};

// 返回值只控制后续形状检查；部分填写的坐标仍会产生诊断，调用方不应把 true 当作完整性证明。
pub(super) fn validate_runtime_plugin_package_coordinate_presence(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) -> bool {
    let fields = RuntimePluginPackageCoordinateFields::from_manifest(package_manifest);
    validate_runtime_plugin_package_coordinate_completeness(&fields, diagnostics);

    fields.declares_any()
}
