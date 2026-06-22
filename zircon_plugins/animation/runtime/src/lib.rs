pub const PLUGIN_ID: &str = "animation";
pub const PLUGIN_RUNTIME_MODULE_NAME: &str = "animation.runtime";
mod capability;
mod runtime_system;

pub use capability::{
    ANIMATION_RUNTIME_CAPABILITY, ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use runtime_system::{
    register_runtime_system, AnimationRuntimeSystem, ANIMATION_EVALUATE_SYSTEM,
    ANIMATION_SYSTEM_SET,
};
pub use zircon_runtime::animation::{
    apply_sequence_to_world, module_descriptor, sample_clip_events, AnimationClipEvent,
    AnimationDriver, AnimationModule, DefaultAnimationManager, ANIMATION_DRIVER_NAME,
    ANIMATION_MODULE_NAME, ANIMATION_PLAYBACK_CONFIG_KEY, DEFAULT_ANIMATION_MANAGER_NAME,
};
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

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
            .module(PLUGIN_RUNTIME_MODULE_NAME, module_descriptor())?;
        register_runtime_system(&mut module)
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Animation",
        zircon_runtime::builtin::RuntimePluginId::Animation,
        "zircon_plugin_animation_runtime",
    )
    .with_category("runtime")
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(ANIMATION_RUNTIME_CAPABILITY)
    .with_capability(ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY)
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Beta)
    .with_capability_status(
        zircon_runtime::plugin::CapabilityStatusManifest::new(
            ANIMATION_RUNTIME_CAPABILITY,
            zircon_runtime::plugin::CapabilityStatus::Partial,
        )
        .with_bevy_reference("dev/bevy/crates/bevy_animation/src/lib.rs"),
    )
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .with_system_sets([ANIMATION_SYSTEM_SET])
    .with_system_anchors([ANIMATION_EVALUATE_SYSTEM])
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(AnimationRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
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
        assert!(report
            .extensions
            .plugin_runtime_systems()
            .any(|(owner, system)| {
                report.extensions.plugin_module_name(owner) == Some(PLUGIN_RUNTIME_MODULE_NAME)
                    && system.id == ANIMATION_EVALUATE_SYSTEM
                    && system.stage == zircon_runtime::scene::SystemStage::PostUpdate
            }));
        assert_eq!(
            report.package_manifest.modules[0].system_sets,
            vec![ANIMATION_SYSTEM_SET.to_string()]
        );
        assert_eq!(
            report.package_manifest.modules[0].system_anchors,
            vec![ANIMATION_EVALUATE_SYSTEM.to_string()]
        );
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::builtin::RuntimeTargetMode::ServerRuntime,
                zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
            ]
        );
        assert_eq!(report.package_manifest.category, "runtime");
        assert_eq!(
            report.package_manifest.maturity,
            zircon_runtime::plugin::PluginMaturity::Beta
        );
        assert!(report
            .package_manifest
            .capability_statuses
            .iter()
            .any(|status| {
                status.capability == "runtime.plugin.animation"
                    && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
                    && status
                        .bevy_references
                        .iter()
                        .any(|reference| reference == "dev/bevy/crates/bevy_animation/src/lib.rs")
            }));
        assert!(report
            .package_manifest
            .capability_statuses
            .iter()
            .any(|status| {
                status.capability == "runtime.feature.animation.timeline_event_track"
                    && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
            }));
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
