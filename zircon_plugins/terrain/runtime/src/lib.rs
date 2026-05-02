pub const PLUGIN_ID: &str = "terrain";
pub const TERRAIN_COMPONENT_TYPE: &str = "terrain.Component.Terrain";
pub const TERRAIN_HEIGHTFIELD_IMPORTER_ID: &str = "terrain.heightfield";

#[derive(Clone, Debug)]
pub struct TerrainRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl TerrainRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for TerrainRuntimePlugin {
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
            "TerrainPlugin",
            "Terrain runtime plugin",
        ))?;
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

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Terrain",
        zircon_runtime::RuntimePluginId::Terrain,
        "zircon_plugin_terrain_runtime",
    )
    .with_category("authoring")
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.terrain")
}

pub fn terrain_component_descriptor() -> zircon_runtime::plugin::ComponentTypeDescriptor {
    zircon_runtime::plugin::ComponentTypeDescriptor::new(TERRAIN_COMPONENT_TYPE, PLUGIN_ID, "Terrain")
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
    .with_required_capabilities(["runtime.plugin.terrain"])]
}

pub fn runtime_plugin() -> TerrainRuntimePlugin {
    TerrainRuntimePlugin::new()
}

pub fn runtime_package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    let mut manifest = zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
        .with_component(terrain_component_descriptor());
    for importer in terrain_importer_descriptors() {
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
    fn terrain_runtime_plugin_contributes_component_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .components()
            .iter()
            .any(|component| component.type_id == TERRAIN_COMPONENT_TYPE));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 1);
        assert_eq!(
            report.package_manifest.asset_importers[0].source_extensions,
            vec!["raw".to_string(), "r16".to_string(), "png".to_string()]
        );
    }
}
