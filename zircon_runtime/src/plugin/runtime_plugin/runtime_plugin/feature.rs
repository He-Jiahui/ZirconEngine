use crate::plugin::{
    PluginFeatureBundleManifest, PluginFinishContext, PluginReadyContext, PluginRuntimeContext,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};

pub trait RuntimePluginFeature {
    fn manifest(&self) -> PluginFeatureBundleManifest;

    fn register(
        &self,
        _registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }

    fn ready(
        &self,
        _context: &PluginReadyContext<'_>,
    ) -> Result<bool, RuntimeExtensionRegistryError> {
        Ok(true)
    }

    fn finish(
        &self,
        _context: &mut PluginFinishContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }

    fn activate(
        &self,
        _context: &mut PluginRuntimeContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }

    fn deactivate(&self, _context: &mut PluginRuntimeContext<'_>) {}
}
