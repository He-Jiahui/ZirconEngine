use crate::capability::{
    PLUGIN_ID, PREFAB_TOOLS_DECLARATION, PREFAB_TOOLS_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
    RUNTIME_CRATE_NAME,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginDistributionManifest, PluginModuleManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

pub const PREFAB_TOOLS_DIST_CRATE_NAME: &str = "zircon_plugin_prefab_tools_dist";
pub const PREFAB_TOOLS_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_prefab_tools_runtime_entry_v3";
pub const PREFAB_INSTANCE_COMPONENT_TYPE: &str = "prefab_tools.Component.PrefabInstance";
pub const PREFAB_IMPORTER_ID: &str = "prefab_tools.prefab";

const PREFAB_TOOLS_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct PrefabToolsRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl PrefabToolsRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for PrefabToolsRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for PrefabToolsRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        runtime_package_manifest()
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_component(prefab_instance_component_descriptor())?;
        for importer in prefab_importer_descriptors() {
            registry.register_asset_importer(
                zircon_runtime::asset::DiagnosticOnlyAssetImporter::new(
                    importer,
                    "prefab importer backend is not installed",
                ),
            )?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    PREFAB_TOOLS_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .with_capability_status(CapabilityStatusManifest::new(
            PREFAB_TOOLS_RUNTIME_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .into_descriptor()
}

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        "prefab_tools.runtime",
        "Prefab tools runtime plugin",
    )
}

pub fn prefab_instance_component_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(PREFAB_INSTANCE_COMPONENT_TYPE, PLUGIN_ID, "Prefab Instance")
        .with_property("prefab", "asset_ref", true)
        .with_property("overrides", "json", false)
}

pub fn prefab_importer_descriptors() -> Vec<zircon_runtime::asset::AssetImporterDescriptor> {
    vec![
        zircon_runtime::asset::AssetImporterDescriptor::new(
            PREFAB_IMPORTER_ID,
            PLUGIN_ID,
            zircon_runtime::asset::AssetKind::Prefab,
            1,
        )
        .with_full_suffixes([".prefab.toml"])
        .with_required_capabilities([PREFAB_TOOLS_RUNTIME_CAPABILITY]),
    ]
}

zircon_plugin_sdk::runtime_plugin_exports!(PrefabToolsRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

pub fn runtime_package_manifest() -> PluginPackageManifest {
    let mut manifest = runtime_plugin_descriptor()
        .package_manifest()
        .with_component(prefab_instance_component_descriptor());
    manifest = manifest.with_native_module(
        PluginModuleManifest::native("prefab_tools.dist", PREFAB_TOOLS_DIST_CRATE_NAME)
            .with_target_modes([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
    );
    for importer in prefab_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest.with_distribution(PluginDistributionManifest {
        forms: vec!["dist".to_string()],
        default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
        abi_version: Some(NATIVE_ABI_VERSION_V3),
        engine_compat: PREFAB_TOOLS_DIST_ENGINE_COMPAT.to_string(),
        dist_crate: PREFAB_TOOLS_DIST_CRATE_NAME.to_string(),
        descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
        runtime_entry: PREFAB_TOOLS_DIST_RUNTIME_ENTRY.to_string(),
        ..PluginDistributionManifest::default()
    })
}
