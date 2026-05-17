mod builtin_modules;
mod engine_entry;
mod entry_config;
mod entry_profile;
mod entry_runner;
mod first_party_runtime_plugins;
#[cfg(feature = "platform-winit")]
mod runtime_entry_app;
pub(crate) mod runtime_library;

#[cfg(test)]
mod tests;

pub use engine_entry::{
    BuiltinEngineEntry, EngineEntry, EntryModuleSelection, EntryModuleSelectionReport, EntryRunMode,
};
pub use entry_config::EntryConfig;
pub use entry_profile::EntryProfile;
pub use entry_runner::{EntryRunner, NativePluginRuntimeBootstrap};
pub use first_party_runtime_plugins::{
    first_party_runtime_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_runtime_profile,
};
