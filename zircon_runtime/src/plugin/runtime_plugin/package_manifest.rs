use crate::{plugin::PluginModuleManifest, plugin::PluginPackageManifest};

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest =
            PluginPackageManifest::new(self.package_id.clone(), self.display_name.clone())
                .with_category(self.category.clone())
                .with_supported_targets(self.target_modes.iter().copied())
                .with_runtime_module(
                    PluginModuleManifest::runtime(
                        format!("{}.runtime", self.package_id),
                        self.crate_name.clone(),
                    )
                    .with_target_modes(self.target_modes.iter().copied())
                    .with_capabilities(self.capabilities.iter().cloned()),
                );
        for capability in &self.capabilities {
            manifest = manifest.with_capability(capability.clone());
        }
        for feature in &self.optional_features {
            manifest = manifest.with_optional_feature(feature.clone());
        }
        manifest.default_packaging = self.default_packaging.clone();
        manifest
    }
}
