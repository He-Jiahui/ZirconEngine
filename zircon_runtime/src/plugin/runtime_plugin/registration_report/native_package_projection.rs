mod crates;
mod target_modes;

use crate::plugin::{ExportPackagingStrategy, PluginPackageManifest, ProjectPluginSelection};

use self::crates::{native_package_editor_crate, native_package_runtime_crate};
use self::target_modes::native_package_target_modes;

pub(in crate::plugin::runtime_plugin::registration_report) fn native_project_selection_from_package(
    package_manifest: &PluginPackageManifest,
) -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: package_manifest.id.clone(),
        enabled: true,
        required: false,
        target_modes: native_package_target_modes(package_manifest),
        packaging: ExportPackagingStrategy::NativeDynamic,
        runtime_crate: native_package_runtime_crate(package_manifest),
        editor_crate: native_package_editor_crate(package_manifest),
        features: Vec::new(),
    }
}
