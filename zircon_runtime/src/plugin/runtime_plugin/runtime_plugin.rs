use crate::plugin::{
    PluginFeatureBundleManifest, PluginPackageManifest, ProjectPluginSelection,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};

use super::RuntimePluginDescriptor;

pub trait RuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor;

    fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor().package_manifest()
    }

    fn project_selection(&self) -> ProjectPluginSelection {
        self.descriptor().project_selection()
    }

    fn register_runtime_extensions(
        &self,
        _registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }
}

pub trait RuntimePluginFeature {
    fn manifest(&self) -> PluginFeatureBundleManifest;

    fn register_runtime_extensions(
        &self,
        _registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }
}

impl RuntimePlugin for RuntimePluginDescriptor {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        self
    }
}
