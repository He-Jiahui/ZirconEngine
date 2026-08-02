use crate::core::framework::project::ProjectPluginSelection;
use crate::core::{ModuleDescriptor, ModuleLifecycle};
use crate::plugin::{
    PluginPackageManifest, PluginShaderModuleSource, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError,
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

    /// Returns module text resolved by a linked plugin before renderer construction.
    /// Native packages populate the same registry from their package manifest files.
    fn shader_module_sources(&self) -> Vec<PluginShaderModuleSource> {
        Vec::new()
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
