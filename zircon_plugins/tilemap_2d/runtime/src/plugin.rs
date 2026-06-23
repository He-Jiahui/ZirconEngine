use crate::capability::{RUNTIME_CAPABILITIES, TILEMAP_2D_RUNTIME_CAPABILITY};

pub const PLUGIN_ID: &str = "tilemap_2d";
pub const TILEMAP_COMPONENT_TYPE: &str = "tilemap_2d.Component.TileMap";
pub const TILED_IMPORTER_ID: &str = "tilemap_2d.tiled";

#[derive(Clone, Debug)]
pub struct Tilemap2dRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl Tilemap2dRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for Tilemap2dRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
        runtime_package_manifest()
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
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

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Tilemap 2D",
        zircon_runtime::builtin::RuntimePluginId::Tilemap2d,
        "zircon_plugin_tilemap_2d_runtime",
    )
    .with_category("authoring")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Beta)
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(TILEMAP_2D_RUNTIME_CAPABILITY)
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        TILEMAP_2D_RUNTIME_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .build()
}

pub fn tilemap_component_descriptor() -> zircon_runtime::plugin::ComponentTypeDescriptor {
    zircon_runtime::plugin::ComponentTypeDescriptor::new(
        TILEMAP_COMPONENT_TYPE,
        PLUGIN_ID,
        "Tilemap 2D",
    )
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

pub fn runtime_package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    let mut manifest = runtime_plugin_descriptor()
        .package_manifest()
        .with_component(tilemap_component_descriptor());
    for importer in tilemap_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}
