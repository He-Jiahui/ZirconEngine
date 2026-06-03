use crate::plugin::{
    PluginFeatureBundleManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};

pub trait RuntimePluginFeature {
    fn manifest(&self) -> PluginFeatureBundleManifest;

    fn register_runtime_extensions(
        &self,
        _registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }
}
