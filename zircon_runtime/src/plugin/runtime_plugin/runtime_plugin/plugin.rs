use crate::plugin::{
    PluginFinishContext, PluginPackageManifest, PluginRuntimeContext, ProjectPluginSelection,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};

use super::super::RuntimePluginDescriptor;

pub trait RuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor;

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

    fn register_runtime_extensions(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.register(registry)
    }
}

impl RuntimePlugin for RuntimePluginDescriptor {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        self
    }
}
