use crate::capability::{RUNTIME_CAPABILITIES, TILEMAP_2D_RUNTIME_CAPABILITY};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, ComponentTypeDescriptor, ExportPackagingStrategy,
    PluginDistributionManifest, PluginMaturity, PluginModuleManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

pub const PLUGIN_ID: &str = "tilemap_2d";
pub const TILEMAP_2D_DIST_CRATE_NAME: &str = "zircon_plugin_tilemap_2d_dist";
pub const TILEMAP_2D_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_tilemap_2d_runtime_entry_v3";
pub const TILEMAP_COMPONENT_TYPE: &str = "tilemap_2d.Component.TileMap";
pub const TILED_IMPORTER_ID: &str = "tilemap_2d.tiled";

const TILEMAP_2D_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct Tilemap2dRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl Tilemap2dRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for Tilemap2dRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for Tilemap2dRuntimePlugin {
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
        registry.register_module(zircon_runtime::core::ModuleDescriptor::new(
            "Tilemap2dPlugin",
            "Tilemap 2D runtime plugin",
        ))?;
        registry.register_component(tilemap_component_descriptor())?;
        for importer in tilemap_importer_descriptors() {
            registry.register_asset_importer(
                zircon_runtime::asset::DiagnosticOnlyAssetImporter::new(
                    importer,
                    "Tiled tilemap importer backend is not installed",
                ),
            )?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Tilemap 2D",
        RuntimePluginId::Tilemap2d,
        "zircon_plugin_tilemap_2d_runtime",
    )
    .with_category("authoring")
    .with_maturity(PluginMaturity::Beta)
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability(TILEMAP_2D_RUNTIME_CAPABILITY)
    .with_capability_status(CapabilityStatusManifest::new(
        TILEMAP_2D_RUNTIME_CAPABILITY,
        CapabilityStatus::Partial,
    ))
    .build()
}

pub fn tilemap_component_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(TILEMAP_COMPONENT_TYPE, PLUGIN_ID, "Tilemap 2D")
        .with_property("tilemap", "asset_ref", true)
        .with_property("material", "asset_ref", false)
}

pub fn tilemap_importer_descriptors() -> Vec<zircon_runtime::asset::AssetImporterDescriptor> {
    vec![zircon_runtime::asset::AssetImporterDescriptor::new(
        TILED_IMPORTER_ID,
        PLUGIN_ID,
        zircon_runtime::asset::AssetKind::TileMap,
        1,
    )
    .with_source_extensions(["tmx", "tsx", "json"])
    .with_required_capabilities([TILEMAP_2D_RUNTIME_CAPABILITY])]
}

zircon_plugin_sdk::runtime_plugin_exports!(Tilemap2dRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

pub fn runtime_package_manifest() -> PluginPackageManifest {
    let mut manifest = runtime_plugin_descriptor()
        .package_manifest()
        .with_component(tilemap_component_descriptor());
    manifest
        .default_packaging
        .push(ExportPackagingStrategy::NativeDynamic);
    manifest = manifest.with_native_module(
        PluginModuleManifest::native("tilemap_2d.dist", TILEMAP_2D_DIST_CRATE_NAME)
            .with_target_modes([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
    );
    for importer in tilemap_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest.with_distribution(PluginDistributionManifest {
        forms: vec!["dist".to_string()],
        default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
        abi_version: Some(NATIVE_ABI_VERSION_V3),
        engine_compat: TILEMAP_2D_DIST_ENGINE_COMPAT.to_string(),
        dist_crate: TILEMAP_2D_DIST_CRATE_NAME.to_string(),
        descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
        runtime_entry: TILEMAP_2D_DIST_RUNTIME_ENTRY.to_string(),
        ..PluginDistributionManifest::default()
    })
}
