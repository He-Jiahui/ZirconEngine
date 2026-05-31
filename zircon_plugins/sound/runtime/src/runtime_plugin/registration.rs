use crate::components::sound_component_descriptors;
use crate::module::module_descriptor;
use crate::package::attach::attach_sound_manifest_contributions;
use crate::package::events::sound_event_catalogs;
use crate::package::options::sound_options;

use super::descriptor::runtime_plugin_descriptor;

#[derive(Clone, Debug)]
pub struct SoundRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl SoundRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for SoundRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
        attach_sound_manifest_contributions(self.descriptor.package_manifest())
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        for component in sound_component_descriptors() {
            registry.register_component(component)?;
        }
        for option in sound_options() {
            registry.register_plugin_option(option)?;
        }
        for event_catalog in sound_event_catalogs() {
            registry.register_plugin_event_catalog(event_catalog)?;
        }
        Ok(())
    }
}

pub fn runtime_plugin() -> SoundRuntimePlugin {
    SoundRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &["runtime.plugin.sound"]
}
