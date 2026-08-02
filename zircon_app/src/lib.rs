//! Entry runners that bootstrap the core runtime and host editor/runtime shells.

mod entry;
pub mod plugins;
pub mod prelude;
#[cfg(feature = "platform-window")]
mod runtime_presenter;

#[cfg(feature = "target-editor-host")]
pub use entry::EditorApplicationComposition;
pub use entry::{
    bootstrap_export_runtime, bootstrap_export_runtime_with_native_plugins_from_export_root,
    bootstrap_export_runtime_with_report, discover_export_root, ExportRuntimeBootstrapConfig,
    ExportRuntimePluginFeatureRegistrationProvider, ExportRuntimePluginRegistrationProvider,
};
pub use entry::{
    first_party_runtime_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_runtime_profile,
};
pub use entry::{
    BuiltinEngineEntry, EngineEntry, EntryModuleSelection, EntryModuleSelectionReport, EntryRunMode,
};
pub use entry::{
    EntryConfig, EntryProfile, EntryRunner, EntryRuntimeBootstrap, NativePluginRuntimeBootstrap,
};
pub use plugins::{
    DefaultPlugins, DevPlugins, HeadlessPlugins, MinimalPlugins, PluginGroup, PluginGroupBuilder,
    PluginGroupError, ResolvedPluginGroup,
};

#[cfg(test)]
mod tests;
