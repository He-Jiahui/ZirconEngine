use std::sync::Arc;

use zircon_runtime::core::{
    ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{factory, qualified_name};

pub const PLUGIN_ID: &str = "texture";
pub const TEXTURE_MODULE_NAME: &str = "TextureModule";
pub const TEXTURE_MANAGER_NAME: &str = "TextureModule.Manager.TextureManager";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureImportSummary {
    pub width: u32,
    pub height: u32,
    pub mip_count: u32,
    pub texel_count: u64,
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextureManager;

impl DefaultTextureManager {
    pub fn summarize_texture(
        &self,
        width: u32,
        height: u32,
        mip_count: u32,
    ) -> TextureImportSummary {
        TextureImportSummary {
            width,
            height,
            mip_count: mip_count.max(1),
            texel_count: u64::from(width).saturating_mul(u64::from(height)),
        }
    }
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        TEXTURE_MODULE_NAME,
        "Texture import and runtime metadata plugin",
    )
    .with_manager(ManagerDescriptor::new(
        qualified_name(TEXTURE_MODULE_NAME, ServiceKind::Manager, "TextureManager"),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| Ok(Arc::new(DefaultTextureManager) as ServiceObject)),
    ))
}

#[derive(Clone, Debug)]
pub struct TextureRuntimePlugin {
    descriptor: zircon_runtime::RuntimePluginDescriptor,
}

impl TextureRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::RuntimePlugin for TextureRuntimePlugin {
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
        "Texture",
        zircon_runtime::RuntimePluginId::Texture,
        "zircon_plugin_texture_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.texture")
}

pub fn runtime_plugin() -> TextureRuntimePlugin {
    TextureRuntimePlugin::new()
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
    &["runtime.plugin.texture"]
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::CoreRuntime;

    use super::*;

    #[test]
    fn texture_registration_contributes_runtime_module() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == TEXTURE_MODULE_NAME));
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::RuntimeTargetMode::EditorHost,
            ]
        );
    }

    #[test]
    fn texture_module_resolves_manager_and_summarizes_texture() {
        let runtime = CoreRuntime::new();
        runtime.register_module(module_descriptor()).unwrap();
        runtime.activate_module(TEXTURE_MODULE_NAME).unwrap();
        let manager = runtime
            .handle()
            .resolve_manager::<DefaultTextureManager>(TEXTURE_MANAGER_NAME)
            .unwrap();

        let summary = manager.summarize_texture(16, 8, 0);

        assert_eq!(summary.mip_count, 1);
        assert_eq!(summary.texel_count, 128);
    }
}
