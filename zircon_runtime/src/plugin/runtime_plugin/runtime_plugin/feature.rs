use crate::plugin::{
    PluginFeatureBundleManifest, PluginFinishContext, PluginRuntimeContext,
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
