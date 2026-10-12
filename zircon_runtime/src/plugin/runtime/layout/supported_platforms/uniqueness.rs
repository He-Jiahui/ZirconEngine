use crate::core::framework::project::ExportTargetPlatform;

pub(super) fn validate_runtime_plugin_package_supported_platform_uniqueness(
    platform: ExportTargetPlatform,
    seen: &mut u8,
    diagnostics: &mut Vec<String>,
) {
    let platform_bit = match platform {
        ExportTargetPlatform::Windows => 0b0000_0001,
        ExportTargetPlatform::Linux => 0b0000_0010,
        ExportTargetPlatform::Macos => 0b0000_0100,
        ExportTargetPlatform::Android => 0b0000_1000,
        ExportTargetPlatform::Ios => 0b0001_0000,
        ExportTargetPlatform::WebGpu => 0b0010_0000,
        ExportTargetPlatform::Wasm => 0b0100_0000,
        ExportTargetPlatform::Headless => 0b1000_0000,
    };
    if *seen & platform_bit != 0 {
        diagnostics.push(format!(
            "runtime plugin package manifest supported_platforms platform {platform:?} must be unique"
        ));
    } else {
        *seen |= platform_bit;
    }
}
