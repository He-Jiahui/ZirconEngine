use crate::capability::{RUNTIME_CAPABILITIES, TERRAIN_RUNTIME_CAPABILITY};
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginDistributionManifest, PluginMaturity,
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

pub const PLUGIN_ID: &str = "terrain";
pub const TERRAIN_DIST_CRATE_NAME: &str = "zircon_plugin_terrain_dist";
pub const TERRAIN_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_terrain_runtime_entry_v3";
pub const TERRAIN_COMPONENT_TYPE: &str = "terrain.Component.Terrain";
pub const TERRAIN_HEIGHTFIELD_IMPORTER_ID: &str = "terrain.heightfield";

const TERRAIN_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct TerrainRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl TerrainRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for TerrainRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for TerrainRuntimePlugin {
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
        registry.register_component(terrain_component_descriptor())?;
        for importer in terrain_importer_descriptors() {
            registry.register_asset_importer(
                zircon_runtime::asset::DiagnosticOnlyAssetImporter::new(
                    importer,
                    "terrain heightfield importer backend is not installed",
                ),
            )?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Terrain",
        RuntimePluginId::Terrain,
        "zircon_plugin_terrain_runtime",
    )
    .with_module_descriptor(module_descriptor())
    .with_category("authoring")
    .with_maturity(PluginMaturity::Beta)
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability(TERRAIN_RUNTIME_CAPABILITY)
    .with_capability_status(CapabilityStatusManifest::new(
        TERRAIN_RUNTIME_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .build()
}

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new("terrain.runtime", "Terrain runtime plugin")
}

pub fn terrain_component_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(TERRAIN_COMPONENT_TYPE, PLUGIN_ID, "Terrain")
        .with_property("terrain", "asset_ref", true)
        .with_property("layers", "asset_ref", false)
}

pub fn terrain_importer_descriptors() -> Vec<zircon_runtime::asset::AssetImporterDescriptor> {
    vec![zircon_runtime::asset::AssetImporterDescriptor::new(
        TERRAIN_HEIGHTFIELD_IMPORTER_ID,
        PLUGIN_ID,
        zircon_runtime::asset::AssetKind::Terrain,
        1,
    )
    .with_source_extensions(["raw", "r16", "png"])
    .with_required_capabilities([TERRAIN_RUNTIME_CAPABILITY])]
}

zircon_plugin_sdk::runtime_plugin_exports!(TerrainRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

pub fn runtime_package_manifest() -> PluginPackageManifest {
    let mut manifest = runtime_plugin_descriptor()
        .package_manifest()
        .with_component(terrain_component_descriptor());
    manifest
        .default_packaging
        .push(ExportPackagingStrategy::NativeDynamic);
    manifest = manifest.with_native_module(
        PluginModuleManifest::native("terrain.dist", TERRAIN_DIST_CRATE_NAME)
            .with_target_modes([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
    );
    for importer in terrain_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest.with_distribution(PluginDistributionManifest {
        forms: vec!["dist".to_string()],
        default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
        abi_version: Some(NATIVE_ABI_VERSION_V3),
        engine_compat: TERRAIN_DIST_ENGINE_COMPAT.to_string(),
        dist_crate: TERRAIN_DIST_CRATE_NAME.to_string(),
        descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
        runtime_entry: TERRAIN_DIST_RUNTIME_ENTRY.to_string(),
        ..PluginDistributionManifest::default()
    })
}
