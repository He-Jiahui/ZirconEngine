pub const PLUGIN_ID: &str = "animation";
pub const ANIMATION_PLAYBACK_CONFIG_KEY: &str = "animation.playback_settings";

mod manager;
mod module;
mod scene_hook;
mod sequence;

pub use manager::DefaultAnimationManager;
pub use module::{
    module_descriptor, AnimationDriver, AnimationModule, ANIMATION_DRIVER_NAME,
    ANIMATION_MODULE_NAME, DEFAULT_ANIMATION_MANAGER_NAME,
};
pub use scene_hook::{scene_hook_registration, AnimationSceneRuntimeHook};
pub use sequence::apply_sequence_to_world;
pub use zircon_runtime::core::framework::animation::AnimationSequenceApplyReport;
pub use zircon_runtime::core::manager::ANIMATION_MANAGER_NAME;

#[derive(Clone, Debug)]
pub struct AnimationRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl AnimationRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for AnimationRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_scene_hook(scene_hook_registration())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Animation",
        zircon_runtime::RuntimePluginId::Animation,
        "zircon_plugin_animation_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.animation")
    .with_capability("runtime.feature.animation.timeline_event_track")
}

pub fn runtime_plugin() -> AnimationRuntimePlugin {
    AnimationRuntimePlugin::new()
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
    &[
        "runtime.plugin.animation",
        "runtime.feature.animation.timeline_event_track",
    ]
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::CoreRuntime;

    use super::*;

    #[test]
    fn animation_registration_contributes_runtime_module() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == ANIMATION_MODULE_NAME));
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::RuntimeTargetMode::ServerRuntime,
                zircon_runtime::RuntimeTargetMode::EditorHost,
            ]
        );
    }

    #[test]
    fn animation_module_resolves_manager() {
        let runtime = CoreRuntime::new();
        runtime.register_module(module_descriptor()).unwrap();
        runtime.activate_module(ANIMATION_MODULE_NAME).unwrap();

        runtime
            .handle()
            .resolve_manager::<DefaultAnimationManager>(DEFAULT_ANIMATION_MANAGER_NAME)
            .unwrap();
    }
}
