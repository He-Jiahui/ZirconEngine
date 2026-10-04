mod rows;
mod runtime_module;

use crate::core::framework::project::ExportTargetPlatform;
use crate::plugin::PluginPackageManifest;

use self::rows::assign_descriptor_package_manifest_rows;
use self::runtime_module::descriptor_runtime_module_manifest;
use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    /// 目标模式沿用描述符声明；包支持的平台单独固定为 Windows、Linux 与 macOS。
    pub fn package_manifest(&self) -> PluginPackageManifest {
        let manifest =
            PluginPackageManifest::new(self.package_id.clone(), self.display_name.clone())
                .with_category(self.category.clone())
                .with_maturity(self.maturity)
                .with_supported_targets(self.target_modes.iter().copied())
                .with_supported_platforms([
                    ExportTargetPlatform::Windows,
                    ExportTargetPlatform::Linux,
                    ExportTargetPlatform::Macos,
                ])
                .with_package_role(self.package_role)
                .with_runtime_module(descriptor_runtime_module_manifest(self));
        assign_descriptor_package_manifest_rows(self, manifest)
    }
}
