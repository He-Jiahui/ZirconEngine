use crate::capability::{
    RUNTIME_CAPABILITIES, VIRTUAL_GEOMETRY_ADVANCED_RENDER_CAPABILITY,
    VIRTUAL_GEOMETRY_RUNTIME_CAPABILITY,
};
use crate::{
    module_descriptor, render_feature_descriptor, render_pass_executor_registrations,
    runtime_prepare_collector_registration, virtual_geometry_runtime_provider_registration,
    PLUGIN_ID,
};

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

impl zircon_runtime::plugin::RuntimePlugin for VirtualGeometryRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
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
