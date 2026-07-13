use crate::core::framework::project::ExportTargetPlatform;

pub(super) fn validate_runtime_plugin_package_supported_platform_uniqueness(
    platform: ExportTargetPlatform,
    seen: &mut Vec<ExportTargetPlatform>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&platform) {
        diagnostics.push(format!(
            "runtime plugin package manifest supported_platforms platform {platform:?} must be unique"
        ));
    } else {
        seen.push(platform);
    }
}
