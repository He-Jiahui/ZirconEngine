mod state;
mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_supported_platform_state;

// 平台集合用于发布目标声明；此处只去重，不推断平台与单个运行模块的可用性。
pub(super) fn validate_runtime_plugin_package_supported_platforms(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_supported_platform_state();
    for platform in package_manifest.supported_platforms.iter().copied() {
        uniqueness::validate_runtime_plugin_package_supported_platform_uniqueness(
            platform,
            &mut seen,
            diagnostics,
        );
    }
}
