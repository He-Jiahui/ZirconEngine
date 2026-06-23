use crate::capability::{PREFAB_TOOLS_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};

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

    fn register(
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
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Prefab Tools",
        zircon_runtime::builtin::RuntimePluginId::PrefabTools,
        "zircon_plugin_prefab_tools_runtime",
    )
    .with_category("authoring")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Beta)
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(PREFAB_TOOLS_RUNTIME_CAPABILITY)
    .with_capability_status(zircon_runtime::plugin::CapabilityStatusManifest::new(
        PREFAB_TOOLS_RUNTIME_CAPABILITY,
        zircon_runtime::plugin::CapabilityStatus::Partial,
    ))
    .build()
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
    .with_required_capabilities([PREFAB_TOOLS_RUNTIME_CAPABILITY])]
}

zircon_plugin_sdk::runtime_plugin_exports!(PrefabToolsRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}

pub fn runtime_package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    let mut manifest = runtime_plugin_descriptor()
        .package_manifest()
        .with_component(prefab_instance_component_descriptor());
    for importer in prefab_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}
