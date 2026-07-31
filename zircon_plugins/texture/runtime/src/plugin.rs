use crate::capability::{
    RUNTIME_CAPABILITIES, TEXTURE_PLUGIN_DECLARATION, TEXTURE_RUNTIME_CRATE_NAME,
};
use crate::module::module_descriptor;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::{
    PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest, RuntimePlugin,
    RuntimePluginDescriptor,
};

pub const TEXTURE_DIST_CRATE_NAME: &str = "zircon_plugin_texture_dist";
pub const TEXTURE_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_texture_runtime_entry_v3";

const TEXTURE_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct TextureRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl TextureRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for TextureRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for TextureRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest.supported_platforms = TEXTURE_PLUGIN_DECLARATION.supported_platforms().to_vec();
        manifest.modules.push(
            PluginModuleManifest::native("texture.dist", TEXTURE_DIST_CRATE_NAME)
                .with_target_modes(TEXTURE_PLUGIN_DECLARATION.target_modes().iter().copied())
                .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
        );
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: TEXTURE_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: TEXTURE_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: TEXTURE_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    TEXTURE_PLUGIN_DECLARATION
        .runtime_declaration(RuntimePluginId::Texture, TEXTURE_RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(TextureRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    TEXTURE_PLUGIN_DECLARATION.capabilities()
}
