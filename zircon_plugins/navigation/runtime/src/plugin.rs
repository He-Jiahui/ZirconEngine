use crate::capability::{
    NAVIGATION_RECAST_CAPABILITY, NAVIGATION_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES,
};
use crate::{
    module_descriptor, navigation_component_descriptors, navigation_event_catalog,
    navigation_plugin_options, PLUGIN_ID,
};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, ExportPackagingStrategy,
    PluginDistributionManifest, PluginMaturity, PluginModuleManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor,
};

pub const NAVIGATION_DIST_CRATE_NAME: &str = "zircon_plugin_navigation_dist";
pub const NAVIGATION_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_navigation_runtime_entry_v3";
const NAVIGATION_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct NavigationRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl NavigationRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for NavigationRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for NavigationRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        for descriptor in navigation_component_descriptors() {
            manifest = manifest.with_component(descriptor);
        }
        for option in navigation_plugin_options() {
            manifest = manifest.with_option(option);
        }
        manifest = manifest.with_event_catalog(navigation_event_catalog());
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("navigation.dist", NAVIGATION_DIST_CRATE_NAME)
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
            engine_compat: NAVIGATION_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: NAVIGATION_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: NAVIGATION_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        for descriptor in navigation_component_descriptors() {
            registry.register_component(descriptor)?;
        }
        for option in navigation_plugin_options() {
            registry.register_plugin_option(option)?;
        }
        registry.register_plugin_event_catalog(navigation_event_catalog())?;
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Navigation",
        RuntimePluginId::Navigation,
        "zircon_plugin_navigation_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability(NAVIGATION_RUNTIME_CAPABILITY)
    .with_capability(NAVIGATION_RECAST_CAPABILITY)
    .with_maturity(PluginMaturity::Beta)
    .with_capability_status(
        CapabilityStatusManifest::new(NAVIGATION_RUNTIME_CAPABILITY, CapabilityStatus::Partial)
            .with_note(
                "Gameplay navmesh/pathfinding is optional; UI navigation parity is separate.",
            ),
    )
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(NavigationRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
