mod rows;
mod runtime_module;

use crate::plugin::PluginPackageManifest;

use self::rows::assign_descriptor_package_manifest_rows;
use self::runtime_module::descriptor_runtime_module_manifest;
use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest =
            PluginPackageManifest::new(self.package_id.clone(), self.display_name.clone())
                .with_category(self.category.clone())
                .with_maturity(self.maturity)
                .with_supported_targets(self.target_modes.iter().copied())
                .with_runtime_module(descriptor_runtime_module_manifest(self));
        assign_descriptor_package_manifest_rows(self, manifest)
    }
}
