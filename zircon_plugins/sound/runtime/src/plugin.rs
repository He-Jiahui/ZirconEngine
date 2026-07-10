use crate::capability::{RUNTIME_CAPABILITIES, SOUND_RUNTIME_CAPABILITY};
use crate::components::sound_component_descriptors;
use crate::package::attach::attach_sound_manifest_contributions;
use crate::package::events::sound_event_catalogs;
use crate::package::options::sound_options;
use crate::runtime_plugin::descriptor::runtime_plugin_descriptor;
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, PluginDistributionManifest, PluginModuleManifest,
    PluginPackageManifest,
};

pub const SOUND_DIST_CRATE_NAME: &str = "zircon_plugin_sound_dist";
pub const SOUND_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_sound_runtime_entry_v3";

const SOUND_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct SoundRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl SoundRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for SoundRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = attach_sound_manifest_contributions(self.descriptor.package_manifest());
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest = manifest.with_native_module(sound_dist_module_manifest());
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: SOUND_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: SOUND_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: SOUND_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        for component in sound_component_descriptors() {
            registry.register_component(component)?;
        }
        for option in sound_options() {
            registry.register_plugin_option(option)?;
        }
        for event_catalog in sound_event_catalogs() {
            registry.register_plugin_event_catalog(event_catalog)?;
        }
        Ok(())
    }
}

pub fn runtime_plugin() -> SoundRuntimePlugin {
    SoundRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn sound_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native("sound.dist", SOUND_DIST_CRATE_NAME)
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(RUNTIME_CAPABILITIES.iter().copied())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

pub fn primary_runtime_capability() -> &'static str {
    SOUND_RUNTIME_CAPABILITY
}
