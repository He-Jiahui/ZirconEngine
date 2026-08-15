use crate::capability::RUNTIME_CRATE_NAME;
use crate::{
    NATIVE_RUNTIME_ENTRY, NEURAL_DECLARATION, NEURAL_MODEL_ASSET_CAPABILITY,
    NEURAL_POST_PROCESS_FEATURE_ID, NEURAL_POST_PROCESS_RUNTIME_CAPABILITY,
    NEURAL_RUNTIME_CAPABILITY, PLUGIN_ID, RENDERING_POST_PROCESS_RUNTIME_CAPABILITY,
    RUNTIME_CAPABILITIES,
};
use zircon_plugin_sdk::{NATIVE_ABI_VERSION_V3, NATIVE_DESCRIPTOR_SYMBOL_V3};
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginDistributionManifest,
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};
use zircon_runtime::{
    core::framework::platform::RuntimeTargetMode, core::framework::project::ExportPackagingStrategy,
};

pub const NEURAL_DIST_CRATE_NAME: &str = "zircon_plugin_neural_dist";
const NEURAL_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";

#[derive(Clone, Debug)]
pub struct NeuralRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl NeuralRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for NeuralRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for NeuralRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let manifest = self.descriptor().package_manifest().with_native_module(
            PluginModuleManifest::native("neural.dist", NEURAL_DIST_CRATE_NAME)
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
            engine_compat: NEURAL_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: NEURAL_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: NATIVE_RUNTIME_ENTRY.name().to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        _registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    NEURAL_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(NEURAL_DECLARATION.module_descriptor())
        .with_capability_status(CapabilityStatusManifest::new(
            NEURAL_RUNTIME_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            NEURAL_MODEL_ASSET_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_optional_feature(neural_post_process_feature_manifest())
        .into_descriptor()
}

pub fn neural_post_process_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        NEURAL_POST_PROCESS_FEATURE_ID,
        "Neural Post Process",
        PLUGIN_ID,
    )
    .with_dependency(PluginFeatureDependency::primary(
        PLUGIN_ID,
        NEURAL_RUNTIME_CAPABILITY,
    ))
    .with_dependency(PluginFeatureDependency::required(
        "rendering",
        RENDERING_POST_PROCESS_RUNTIME_CAPABILITY,
    ))
    .with_capability(NEURAL_POST_PROCESS_RUNTIME_CAPABILITY)
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "neural.post_process.runtime",
            "zircon_plugin_neural_post_process_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([NEURAL_POST_PROCESS_RUNTIME_CAPABILITY]),
    )
    .enabled_by_default(false)
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

zircon_plugin_sdk::runtime_plugin_exports!(NeuralRuntimePlugin);
