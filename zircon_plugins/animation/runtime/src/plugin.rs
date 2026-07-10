use crate::capability::{
    ANIMATION_RUNTIME_CAPABILITY, ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY, RUNTIME_CAPABILITIES,
};
use crate::runtime_system::{
    register_runtime_system, ANIMATION_EVALUATE_SYSTEM, ANIMATION_SYSTEM_SET,
};
use crate::{module_descriptor, PLUGIN_ID};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, ExportPackagingStrategy,
    PluginDistributionManifest, PluginMaturity, PluginModuleManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

pub const PLUGIN_RUNTIME_MODULE_NAME: &str = "animation.runtime";
pub const ANIMATION_DIST_CRATE_NAME: &str = "zircon_plugin_animation_dist";
pub const ANIMATION_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_animation_runtime_entry_v3";

const ANIMATION_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct AnimationRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl AnimationRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for AnimationRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for AnimationRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("animation.dist", ANIMATION_DIST_CRATE_NAME)
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
        );
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: ANIMATION_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: ANIMATION_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: ANIMATION_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
            .module(PLUGIN_RUNTIME_MODULE_NAME)?;
        register_runtime_system(&mut module)
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Animation",
        RuntimePluginId::Animation,
        "zircon_plugin_animation_runtime",
    )
    .with_module_descriptor(module_descriptor())
    .with_category("runtime")
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability(ANIMATION_RUNTIME_CAPABILITY)
    .with_capability(ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY)
    .with_maturity(PluginMaturity::Beta)
    .with_capability_status(
        CapabilityStatusManifest::new(ANIMATION_RUNTIME_CAPABILITY, CapabilityStatus::Partial)
            .with_bevy_reference("dev/bevy/crates/bevy_animation/src/lib.rs"),
    )
    .with_capability_status(CapabilityStatusManifest::new(
        ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .with_system_sets([ANIMATION_SYSTEM_SET])
    .with_system_anchors([ANIMATION_EVALUATE_SYSTEM])
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(AnimationRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
