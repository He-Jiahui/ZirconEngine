use crate::capability::{
    HYBRID_GI_ADVANCED_RENDER_CAPABILITY, HYBRID_GI_DECLARATION, HYBRID_GI_RUNTIME_CAPABILITY,
    RUNTIME_CAPABILITIES, RUNTIME_CRATE_NAME,
};
use crate::{
    hybrid_gi_runtime_provider_registration, module_descriptor, render_feature_descriptor,
    render_pass_executor_registrations, runtime_prepare_collector_registration,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::{
    PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest,
};

#[derive(Clone, Debug)]
pub struct HybridGiRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

pub const HYBRID_GI_DIST_CRATE_NAME: &str = "zircon_plugin_hybrid_gi_dist";
pub const HYBRID_GI_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_hybrid_gi_runtime_entry_v3";

const HYBRID_GI_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

impl HybridGiRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for HybridGiRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_runtime::plugin::RuntimePlugin for HybridGiRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("hybrid_gi.dist", HYBRID_GI_DIST_CRATE_NAME)
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
        );
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: HYBRID_GI_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: HYBRID_GI_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: HYBRID_GI_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_render_feature(render_feature_descriptor())?;
        registry.register_hybrid_gi_runtime_provider(hybrid_gi_runtime_provider_registration())?;
        for registration in render_pass_executor_registrations() {
            registry.register_render_pass_executor(registration)?;
        }
        registry.register_runtime_prepare_collector(runtime_prepare_collector_registration())?;
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    HYBRID_GI_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(HybridGiRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
