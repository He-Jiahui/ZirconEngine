mod crates;
mod target_modes;

use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginSelection};
use crate::plugin::PluginPackageManifest;

use self::crates::{native_package_editor_crate, native_package_runtime_crate};
use self::target_modes::native_package_target_modes;

// 原生包默认作为启用的动态包进入选择清单；模块目标与 crate 名从包清单投影。
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
