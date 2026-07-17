mod builtin_modules;
#[cfg(feature = "target-editor-host")]
pub(crate) mod cli;
mod engine_entry;
mod entry_config;
mod entry_profile;
mod entry_runner;
mod export_bootstrap;
#[cfg(feature = "target-editor-host")]
mod first_party_editor_plugins;
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
pub use entry_runner::{EntryRunner, EntryRuntimeBootstrap, NativePluginRuntimeBootstrap};
pub use export_bootstrap::{
    bootstrap_export_runtime, bootstrap_export_runtime_with_native_plugins_from_export_root,
    bootstrap_export_runtime_with_report, discover_export_root, ExportRuntimeBootstrapConfig,
    ExportRuntimePluginFeatureRegistrationProvider, ExportRuntimePluginRegistrationProvider,
};
#[cfg(feature = "target-editor-host")]
pub use first_party_editor_plugins::{
    first_party_editor_plugin_registrations_for_config,
    first_party_editor_plugin_registrations_for_manifest,
};
pub use first_party_runtime_plugins::{
    first_party_runtime_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_runtime_profile,
};
