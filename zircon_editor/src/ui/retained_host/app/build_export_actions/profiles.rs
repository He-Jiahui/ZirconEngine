use std::path::{Path, PathBuf};

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ExportProfile, ExportTargetPlatform, RuntimeProfileId,
};

pub(in crate::ui::retained_host::app) fn desktop_export_profiles() -> Vec<ExportProfile> {
    let desktop = [
        ("desktop_windows", ExportTargetPlatform::Windows),
        ("desktop_linux", ExportTargetPlatform::Linux),
        ("desktop_macos", ExportTargetPlatform::Macos),
    ]
    .into_iter()
    .map(|(name, platform)| {
        ExportProfile::new(
            name,
            RuntimeTargetMode::ClientRuntime,
            platform,
            RuntimeProfileId::Client2d,
        )
        .with_strategies([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ])
    });
    let platform_scaffolds = [
        ("mobile_android", ExportTargetPlatform::Android),
        ("mobile_ios", ExportTargetPlatform::Ios),
        ("browser_webgpu", ExportTargetPlatform::WebGpu),
        ("browser_wasm", ExportTargetPlatform::Wasm),
        ("headless_server", ExportTargetPlatform::Headless),
    ]
    .into_iter()
    .map(|(name, platform)| {
        let (target_mode, runtime_profile_id) = if platform == ExportTargetPlatform::Headless {
            (RuntimeTargetMode::ServerRuntime, RuntimeProfileId::Server)
        } else {
            (RuntimeTargetMode::ClientRuntime, RuntimeProfileId::Client2d)
        };
        ExportProfile::new(name, target_mode, platform, runtime_profile_id).with_strategies([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
        ])
    });

    desktop.chain(platform_scaffolds).collect()
}

pub(in crate::ui::retained_host::app) fn desktop_export_profile(
    profile_name: &str,
) -> Option<ExportProfile> {
    desktop_export_profiles()
        .into_iter()
        .find(|profile| profile.name == profile_name)
}

pub(in crate::ui::retained_host::app) fn export_platform_label(
    platform: ExportTargetPlatform,
) -> &'static str {
    match platform {
        ExportTargetPlatform::Windows => "Windows",
        ExportTargetPlatform::Linux => "Linux",
        ExportTargetPlatform::Macos => "macOS",
        ExportTargetPlatform::Android => "Android",
        ExportTargetPlatform::Ios => "iOS",
        ExportTargetPlatform::WebGpu => "WebGPU",
        ExportTargetPlatform::Wasm => "WASM",
        ExportTargetPlatform::Headless => "Headless",
    }
}

pub(in crate::ui::retained_host::app) fn default_desktop_export_output_root(
    project_root: &Path,
    profile_name: &str,
) -> PathBuf {
    project_root
        .join("Builds")
        .join("zircon")
        .join(profile_name)
}
