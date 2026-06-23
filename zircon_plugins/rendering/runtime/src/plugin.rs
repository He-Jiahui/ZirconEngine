use crate::capability::{RENDERING_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
use crate::{feature_manifest, module_descriptor, PLUGIN_ID, RENDERING_FEATURES};

#[derive(Clone, Debug)]
pub struct RenderingRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl RenderingRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for RenderingRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    let mut builder = zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Rendering",
        zircon_runtime::builtin::RuntimePluginId::Rendering,
        "zircon_plugin_rendering_runtime",
    )
    .with_category("rendering")
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Stable)
    .with_target_modes([
        zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(RENDERING_RUNTIME_CAPABILITY);

    for feature in RENDERING_FEATURES {
        builder = builder.with_optional_feature(feature_manifest(*feature));
    }
    builder.build()
}

zircon_plugin_sdk::runtime_plugin_exports!(RenderingRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
