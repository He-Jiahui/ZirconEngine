mod builtin_modules;
#[cfg(feature = "diagnostic-log")]
pub(crate) mod cli;
mod engine_entry;
mod entry_profile;
mod entry_runner;
mod export_bootstrap;
#[cfg(feature = "first-party-editor-catalog")]
mod first_party_editor_plugins;
mod first_party_runtime_plugins;
mod platform_preferences;
mod product_composition;
mod product_host_config;
pub(crate) mod product_shutdown;
#[cfg(feature = "platform-winit")]
mod runtime_entry_app;
pub(crate) mod runtime_library;

#[cfg(test)]
mod tests;

pub(crate) use engine_entry::{BuiltinEngineEntry, EngineEntry};
pub use engine_entry::{EntryModuleSelection, EntryModuleSelectionReport, EntryRunMode};
pub use entry_profile::EntryProfile;
#[cfg(feature = "target-editor-host")]
pub use entry_runner::EditorApplicationComposition;
pub use entry_runner::EntryRunner;
pub use export_bootstrap::{
    bootstrap_export_runtime, bootstrap_export_runtime_with_native_plugins_from_export_root,
    discover_export_root, ExportRuntimeBootstrapConfig,
    ExportRuntimePluginFeatureRegistrationProvider, ExportRuntimePluginRegistrationProvider,
};
#[cfg(feature = "first-party-editor-catalog")]
pub use first_party_editor_plugins::{
    first_party_editor_plugin_registrations_for_config,
    first_party_editor_plugin_registrations_for_manifest,
};
pub use first_party_runtime_plugins::{
    first_party_runtime_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_runtime_profile,
};
pub use product_composition::{ProductComposition, ProductCompositionRequest};
pub use product_host_config::{
    EntryConfig, ProductArtifactDeliveryStatus, ProductArtifactKind, ProductArtifactManifest,
    ProductCapabilityRequirement, ProductConfigSource, ProductConfigSourceSet, ProductEntryKind,
    ProductHostCapabilityPolicy, ProductHostConfigError, ProductHostConfigProvenance,
    ProductPlatformClass, ProductRoleDescriptor, ProductRoleRequest, ProductRunnerKind,
    ProductRuntimeLinkage, ProductShutdownPolicy, ResolvedProductHostConfig,
};
pub use product_shutdown::{ProductExitClass, ProductProcessExitCode};
