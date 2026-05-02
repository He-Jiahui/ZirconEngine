pub const PLUGIN_ID: &str = "prefab_tools";
pub const PREFAB_INSTANCE_COMPONENT_TYPE: &str = "prefab_tools.Component.PrefabInstance";
pub const PREFAB_IMPORTER_ID: &str = "prefab_tools.prefab";

#[derive(Clone, Debug)]
pub struct PrefabToolsRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl PrefabToolsRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for PrefabToolsRuntimePlugin {
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
            "PrefabToolsPlugin",
            "Prefab tools runtime plugin",
        ))?;
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

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Prefab Tools",
        zircon_runtime::RuntimePluginId::PrefabTools,
        "zircon_plugin_prefab_tools_runtime",
    )
    .with_category("authoring")
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.prefab_tools")
}

pub fn prefab_instance_component_descriptor() -> zircon_runtime::plugin::ComponentTypeDescriptor {
    zircon_runtime::plugin::ComponentTypeDescriptor::new(
        PREFAB_INSTANCE_COMPONENT_TYPE,
        PLUGIN_ID,
        "Prefab Instance",
    )
    .with_property("prefab", "asset_ref", true)
    .with_property("overrides", "json", false)
}

pub fn prefab_importer_descriptors() -> Vec<zircon_runtime::asset::AssetImporterDescriptor> {
    vec![zircon_runtime::asset::AssetImporterDescriptor::new(
        PREFAB_IMPORTER_ID,
        PLUGIN_ID,
        zircon_runtime::asset::AssetKind::Prefab,
        1,
    )
    .with_full_suffixes([".prefab.toml"])
    .with_required_capabilities(["runtime.plugin.prefab_tools"])]
}

pub fn runtime_plugin() -> PrefabToolsRuntimePlugin {
    PrefabToolsRuntimePlugin::new()
}

pub fn runtime_package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    let mut manifest = zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
        .with_component(prefab_instance_component_descriptor());
    for importer in prefab_importer_descriptors() {
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
    fn prefab_runtime_plugin_contributes_component_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .components()
            .iter()
            .any(|component| component.type_id == PREFAB_INSTANCE_COMPONENT_TYPE));
        assert_eq!(
            report.package_manifest.asset_importers[0].full_suffixes,
            vec![".prefab.toml".to_string()]
        );
    }
}
