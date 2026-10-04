use std::sync::Arc;

use zircon_runtime::core::framework::render::SolariRuntimeStatus;

pub const SOLARI_MODULE_NAME: &str = "solari.runtime";
pub const SOLARI_PROVIDER_ID: &str = "plugin.solari.runtime";
pub const SOLARI_UNAVAILABLE_MESSAGE: &str =
    "Solari realtime raytraced lighting pass executor is not implemented yet";

mod capability;
mod plugin;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY,
    SOLARI_CAPABILITY, SOLARI_DECLARATION,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, SolariRuntimePlugin,
};

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

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
