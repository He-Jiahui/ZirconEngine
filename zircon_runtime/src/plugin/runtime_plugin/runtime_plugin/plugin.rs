use crate::core::framework::project::ProjectPluginSelection;
use crate::core::{ModuleDescriptor, ModuleLifecycle};
use crate::plugin::{
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};

use super::super::RuntimePluginDescriptor;

pub trait RuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor;

    fn module_descriptor(&self) -> &ModuleDescriptor {
        self.descriptor().module_descriptor()
    }

    /// Returns the kernel-owned lifecycle attached to the embedded module descriptor.
    fn lifecycle(&self) -> &dyn ModuleLifecycle {
        self.module_descriptor().lifecycle.as_ref()
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
}

impl RuntimePlugin for RuntimePluginDescriptor {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        self
    }
}
