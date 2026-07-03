use crate::core::ModuleDescriptor;
use crate::plugin::{
    PluginFinishContext, PluginPackageManifest, PluginReadyContext, PluginRuntimeContext,
    ProjectPluginSelection, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};

use super::super::RuntimePluginDescriptor;

pub trait RuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor;

    fn module_descriptor(&self) -> &ModuleDescriptor {
        self.descriptor().module_descriptor()
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor().package_manifest()
    }

    fn project_selection(&self) -> ProjectPluginSelection {
        self.descriptor().project_selection()
    }

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

impl RuntimePlugin for RuntimePluginDescriptor {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        self
    }
}
