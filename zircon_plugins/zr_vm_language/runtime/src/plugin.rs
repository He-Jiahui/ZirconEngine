use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, ExportPackagingStrategy,
    PluginDistributionManifest, PluginMaturity, PluginModuleManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

use crate::{
    module_descriptor, PLUGIN_ID, RUNTIME_CAPABILITIES, ZR_VM_LANGUAGE_RUNTIME_CAPABILITY,
    ZR_VM_PROJECT_BACKEND_CAPABILITY,
};

pub const ZR_VM_LANGUAGE_DIST_CRATE_NAME: &str = "zircon_plugin_zr_vm_language_dist";
pub const ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_zr_vm_language_runtime_entry_v3";
const ZR_VM_LANGUAGE_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct ZrVmLanguageRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl ZrVmLanguageRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for ZrVmLanguageRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for ZrVmLanguageRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("zr_vm_language.dist", ZR_VM_LANGUAGE_DIST_CRATE_NAME)
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
            engine_compat: ZR_VM_LANGUAGE_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: ZR_VM_LANGUAGE_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_scene_hook(
            zircon_runtime::script::script_scene_fixed_update_hook_registration(),
        )?;
        registry
            .register_scene_hook(zircon_runtime::script::script_scene_update_hook_registration())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "ZrVM Language",
        RuntimePluginId::ZrVmLanguage,
        "zircon_plugin_zr_vm_language_runtime",
    )
    .with_category("runtime")
    .with_maturity(PluginMaturity::Experimental)
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_enabled_by_default(false)
    .with_capability(ZR_VM_LANGUAGE_RUNTIME_CAPABILITY)
    .with_capability(ZR_VM_PROJECT_BACKEND_CAPABILITY)
    .with_capability_status(CapabilityStatusManifest::new(
        ZR_VM_LANGUAGE_RUNTIME_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .with_capability_status(CapabilityStatusManifest::new(
        ZR_VM_PROJECT_BACKEND_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(ZrVmLanguageRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
