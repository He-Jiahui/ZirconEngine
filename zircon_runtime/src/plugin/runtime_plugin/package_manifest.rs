use crate::{PluginModuleManifest, PluginPackageManifest};

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest =
            PluginPackageManifest::new(self.package_id.clone(), self.display_name.clone())
                .with_runtime_module(
                    PluginModuleManifest::runtime(
                        format!("{}.runtime", self.package_id),
                        self.crate_name.clone(),
                    )
                    .with_target_modes(self.target_modes.iter().copied())
                    .with_capabilities(self.capabilities.iter().cloned()),
                );
        manifest.default_packaging = self.default_packaging.clone();
        manifest
    }
}
