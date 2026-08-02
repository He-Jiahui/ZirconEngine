use crate::capability::{RENDERING_DECLARATION, RUNTIME_CAPABILITIES, RUNTIME_CRATE_NAME};
use crate::{RENDERING_FEATURES, feature_manifest, module_descriptor};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::{
    PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest,
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
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    let mut builder = RENDERING_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor());

    for feature in RENDERING_FEATURES {
        builder = builder.with_optional_feature(feature_manifest(*feature));
    }
    builder.into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(RenderingRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
