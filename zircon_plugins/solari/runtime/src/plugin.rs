use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, ExportPackagingStrategy,
    PluginDistributionManifest, PluginMaturity, PluginModuleManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

use crate::{
    module_descriptor, solari_runtime_provider_registration, PLUGIN_ID, RUNTIME_CAPABILITIES,
    RUNTIME_CAPABILITY, SOLARI_CAPABILITY, SOLARI_UNAVAILABLE_MESSAGE,
};

pub const SOLARI_DIST_CRATE_NAME: &str = "zircon_plugin_solari_dist";
pub const SOLARI_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_solari_runtime_entry_v3";
const SOLARI_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct SolariRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl SolariRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl RuntimePlugin for SolariRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("solari.dist", SOLARI_DIST_CRATE_NAME)
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
            engine_compat: SOLARI_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: SOLARI_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: SOLARI_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_solari_runtime_provider(solari_runtime_provider_registration())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Solari",
        RuntimePluginId::Solari,
        "zircon_plugin_solari_runtime",
    )
    .with_module_descriptor(module_descriptor())
    .with_category("rendering")
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_maturity(PluginMaturity::Experimental)
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(SOLARI_CAPABILITY)
    .with_capability_status(CapabilityStatusManifest::new(
        RUNTIME_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .with_capability_status(
        CapabilityStatusManifest::new(SOLARI_CAPABILITY, CapabilityStatus::Partial)
            .with_note(SOLARI_UNAVAILABLE_MESSAGE),
    )
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(SolariRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
