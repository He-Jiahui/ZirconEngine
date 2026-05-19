use std::sync::Arc;

use zircon_runtime::core::framework::render::SolariRuntimeStatus;

pub const PLUGIN_ID: &str = "solari";
pub const SOLARI_MODULE_NAME: &str = "SolariPluginModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.solari";
pub const SOLARI_CAPABILITY: &str = "runtime.render.experimental.solari";
pub const SOLARI_PROVIDER_ID: &str = "plugin.solari.runtime";
pub const SOLARI_UNAVAILABLE_MESSAGE: &str =
    "Solari realtime raytraced lighting pass executor is not implemented yet";

#[derive(Clone, Debug)]
pub struct SolariRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl SolariRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for SolariRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_solari_runtime_provider(solari_runtime_provider_registration())
    }
}

#[derive(Debug)]
pub struct PluginSolariRuntimeProvider;

impl zircon_runtime::graphics::SolariRuntimeProvider for PluginSolariRuntimeProvider {
    fn runtime_status(&self) -> SolariRuntimeStatus {
        SolariRuntimeStatus::Unavailable
    }

    fn runtime_status_message(&self) -> Option<&str> {
        Some(SOLARI_UNAVAILABLE_MESSAGE)
    }
}

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        SOLARI_MODULE_NAME,
        "Solari experimental render provider contract",
    )
}

pub fn solari_runtime_provider_registration(
) -> zircon_runtime::graphics::SolariRuntimeProviderRegistration {
    zircon_runtime::graphics::SolariRuntimeProviderRegistration::new(
        SOLARI_PROVIDER_ID,
        Arc::new(PluginSolariRuntimeProvider),
    )
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Solari",
        zircon_runtime::RuntimePluginId::Solari,
        "zircon_plugin_solari_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(SOLARI_CAPABILITY)
}

pub fn runtime_plugin() -> SolariRuntimePlugin {
    SolariRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, SOLARI_CAPABILITY]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solari_registration_contributes_unavailable_provider_contract() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == SOLARI_MODULE_NAME));
        assert_eq!(
            report.extensions.solari_runtime_providers()[0].provider_id(),
            SOLARI_PROVIDER_ID
        );
        assert!(report
            .package_manifest
            .capabilities
            .contains(&SOLARI_CAPABILITY.to_string()));

        let provider = &report.extensions.solari_runtime_providers()[0];
        let availability = provider.provider().availability(provider.provider_id());
        assert_eq!(
            availability.provider_status,
            SolariRuntimeStatus::Unavailable
        );
        assert_eq!(
            availability.provider_message.as_deref(),
            Some(SOLARI_UNAVAILABLE_MESSAGE)
        );
    }

    #[test]
    fn solari_runtime_capabilities_match_package_manifest() {
        let manifest = package_manifest();
        for capability in runtime_capabilities() {
            assert!(
                manifest.capabilities.contains(&capability.to_string()),
                "missing {capability}"
            );
            assert!(
                manifest.modules[0]
                    .capabilities
                    .contains(&capability.to_string()),
                "module missing {capability}"
            );
        }
    }
}
