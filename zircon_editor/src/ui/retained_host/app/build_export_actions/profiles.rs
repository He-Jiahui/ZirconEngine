use std::path::{Path, PathBuf};

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ExportProfile, ExportTargetPlatform, RuntimeProfileId,
};

const DESKTOP_EXPORT_PROFILE_NAMES: [&str; 8] = [
    "desktop_windows",
    "desktop_linux",
    "desktop_macos",
    "mobile_android",
    "mobile_ios",
    "browser_webgpu",
    "browser_wasm",
    "headless_server",
];

pub(in crate::ui::retained_host::app) fn desktop_export_profiles() -> Vec<ExportProfile> {
    DESKTOP_EXPORT_PROFILE_NAMES
        .into_iter()
        .map(|profile_name| {
            desktop_export_profile(profile_name).expect("built-in export profile must be defined")
        })
        .collect()
}

pub(in crate::ui::retained_host::app) fn desktop_export_profile(
    profile_name: &str,
) -> Option<ExportProfile> {
    let profile = match profile_name {
        "desktop_windows" => {
            desktop_client_export_profile(profile_name, ExportTargetPlatform::Windows)
        }
        "desktop_linux" => desktop_client_export_profile(profile_name, ExportTargetPlatform::Linux),
        "desktop_macos" => desktop_client_export_profile(profile_name, ExportTargetPlatform::Macos),
        "mobile_android" => {
            client_scaffold_export_profile(profile_name, ExportTargetPlatform::Android)
        }
        "mobile_ios" => client_scaffold_export_profile(profile_name, ExportTargetPlatform::Ios),
        "browser_webgpu" => {
            client_scaffold_export_profile(profile_name, ExportTargetPlatform::WebGpu)
        }
        "browser_wasm" => client_scaffold_export_profile(profile_name, ExportTargetPlatform::Wasm),
        "headless_server" => ExportProfile::new(
            profile_name,
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Headless,
            RuntimeProfileId::Server,
        )
        .with_strategies([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
        ]),
        _ => return None,
    };
    Some(profile)
}

fn desktop_client_export_profile(
    profile_name: &str,
    platform: ExportTargetPlatform,
) -> ExportProfile {
    ExportProfile::new(
        profile_name,
        RuntimeTargetMode::ClientRuntime,
        platform,
        RuntimeProfileId::Client2d,
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::NativeDynamic,
    ])
}

fn client_scaffold_export_profile(
    profile_name: &str,
    platform: ExportTargetPlatform,
) -> ExportProfile {
    ExportProfile::new(
        profile_name,
        RuntimeTargetMode::ClientRuntime,
        platform,
        RuntimeProfileId::Client2d,
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
    ])
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

#[cfg(test)]
#[path = "profiles/direct_lookup_tests.rs"]
mod direct_lookup_tests;
