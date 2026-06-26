use crate::capability::{RENDERING_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
use crate::{feature_manifest, module_descriptor, PLUGIN_ID, RENDERING_FEATURES};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, PluginDistributionManifest, PluginModuleManifest,
    PluginPackageManifest,
};

pub const RENDERING_DIST_CRATE_NAME: &str = "zircon_plugin_rendering_dist";
pub const RENDERING_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_rendering_runtime_entry_v3";

const RENDERING_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct RenderingRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl RenderingRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for RenderingRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_runtime::plugin::RuntimePlugin for RenderingRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("rendering.dist", RENDERING_DIST_CRATE_NAME)
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
            engine_compat: RENDERING_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: RENDERING_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: RENDERING_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    let mut builder = zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Rendering",
        zircon_runtime::builtin::RuntimePluginId::Rendering,
        "zircon_plugin_rendering_runtime",
    )
    .with_category("rendering")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Stable)
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(RENDERING_RUNTIME_CAPABILITY);

    for feature in RENDERING_FEATURES {
        builder = builder.with_optional_feature(feature_manifest(*feature));
    }
    builder.build()
}

zircon_plugin_sdk::runtime_plugin_exports!(RenderingRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
