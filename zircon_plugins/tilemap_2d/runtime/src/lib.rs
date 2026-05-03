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

    fn register_runtime_extensions(
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
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Tilemap 2D",
        zircon_runtime::RuntimePluginId::Tilemap2d,
        "zircon_plugin_tilemap_2d_runtime",
    )
    .with_category("authoring")
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.tilemap_2d")
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
    .with_required_capabilities(["runtime.plugin.tilemap_2d"])]
}

pub fn runtime_plugin() -> Tilemap2dRuntimePlugin {
    Tilemap2dRuntimePlugin::new()
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

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    runtime_package_manifest()
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilemap_runtime_plugin_contributes_component_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .components()
            .iter()
            .any(|component| component.type_id == TILEMAP_COMPONENT_TYPE));
        assert_eq!(
            report.package_manifest.asset_importers[0].source_extensions,
            vec!["tmx".to_string(), "tsx".to_string(), "json".to_string()]
        );
    }
}
