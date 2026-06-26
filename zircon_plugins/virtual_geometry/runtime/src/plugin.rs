use crate::capability::{
    RUNTIME_CAPABILITIES, VIRTUAL_GEOMETRY_ADVANCED_RENDER_CAPABILITY,
    VIRTUAL_GEOMETRY_RUNTIME_CAPABILITY,
};
use crate::{
    module_descriptor, render_feature_descriptor, render_pass_executor_registrations,
    runtime_prepare_collector_registration, virtual_geometry_runtime_provider_registration,
    PLUGIN_ID,
};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, PluginDistributionManifest, PluginModuleManifest,
    PluginPackageManifest,
};

pub const VIRTUAL_GEOMETRY_DIST_CRATE_NAME: &str = "zircon_plugin_virtual_geometry_dist";
pub const VIRTUAL_GEOMETRY_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_virtual_geometry_runtime_entry_v3";

const VIRTUAL_GEOMETRY_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct VirtualGeometryRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl VirtualGeometryRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for VirtualGeometryRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_runtime::plugin::RuntimePlugin for VirtualGeometryRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest
            .default_packaging
            .push(ExportPackagingStrategy::NativeDynamic);
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("virtual_geometry.dist", VIRTUAL_GEOMETRY_DIST_CRATE_NAME)
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
        );
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: VIRTUAL_GEOMETRY_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: VIRTUAL_GEOMETRY_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: VIRTUAL_GEOMETRY_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_render_feature(render_feature_descriptor())?;
        for registration in render_pass_executor_registrations() {
            registry.register_render_pass_executor(registration)?;
        }
        registry.register_runtime_prepare_collector(runtime_prepare_collector_registration())?;
        registry.register_virtual_geometry_runtime_provider(
            virtual_geometry_runtime_provider_registration(),
        )
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Virtual Geometry",
        zircon_runtime::builtin::RuntimePluginId::VirtualGeometry,
        "zircon_plugin_virtual_geometry_runtime",
    )
    .with_category("rendering")
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Experimental)
    .with_capability(VIRTUAL_GEOMETRY_RUNTIME_CAPABILITY)
    .with_capability(VIRTUAL_GEOMETRY_ADVANCED_RENDER_CAPABILITY)
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(VirtualGeometryRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
