pub const PLUGIN_ID: &str = "animation";

mod animation_interface;
mod config;
mod module;
mod sequence_runtime;
mod service_types;

pub use animation_interface::AnimationInterface;
pub use config::AnimationConfig;
pub use module::{
    module_descriptor, AnimationModule, ANIMATION_DRIVER_NAME, ANIMATION_MANAGER_NAME,
    ANIMATION_MODULE_NAME, ANIMATION_PLAYBACK_CONFIG_KEY,
};
pub use sequence_runtime::{apply_sequence_to_world, AnimationSequenceApplyReport};
pub use service_types::{AnimationDriver, DefaultAnimationManager};

#[cfg(test)]
pub(crate) use module::DEFAULT_ANIMATION_MANAGER_NAME;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct AnimationRuntimePlugin {
    descriptor: zircon_runtime::RuntimePluginDescriptor,
}

impl AnimationRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::RuntimePlugin for AnimationRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::RuntimePluginDescriptor {
    zircon_runtime::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Animation",
        zircon_runtime::RuntimePluginId::Animation,
        "zircon_plugin_animation_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.animation")
}

pub fn runtime_plugin() -> AnimationRuntimePlugin {
    AnimationRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_runtime::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::ProjectPluginSelection {
    zircon_runtime::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::RuntimePluginRegistrationReport {
    zircon_runtime::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &["runtime.plugin.animation"]
}
