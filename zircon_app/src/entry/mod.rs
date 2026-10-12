#[path = "builtin_modules.rs"]
mod builtin_modules;
#[cfg(feature = "diagnostic-log")]
#[path = "cli/mod.rs"]
pub(crate) mod cli;
#[path = "engine_entry.rs"]
mod engine_entry;
#[path = "entry_profile.rs"]
mod entry_profile;
#[path = "entry_runner/mod.rs"]
mod entry_runner;
#[path = "export_bootstrap.rs"]
mod export_bootstrap;
#[cfg(feature = "first-party-editor-catalog")]
#[path = "first_party_editor_plugins.rs"]
mod first_party_editor_plugins;
#[path = "first_party_runtime_plugins.rs"]
mod first_party_runtime_plugins;
#[path = "platform_preferences.rs"]
mod platform_preferences;
#[path = "product_composition/mod.rs"]
mod product_composition;
#[path = "product_host_config/mod.rs"]
mod product_host_config;
#[path = "product_shutdown/mod.rs"]
pub(crate) mod product_shutdown;
#[cfg(feature = "platform-winit")]
#[path = "runtime_entry_app/mod.rs"]
mod runtime_entry_app;
#[path = "runtime_library/mod.rs"]
pub(crate) mod runtime_library;
pub use runtime_library::{retry_runtime_startup_cleanup, RuntimeSessionCreateFailure};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

pub(crate) use engine_entry::{BuiltinEngineEntry, EngineEntry};
pub use engine_entry::{EntryModuleSelection, EntryModuleSelectionReport, EntryRunMode};
pub use entry_profile::EntryProfile;
#[cfg(feature = "target-editor-host")]
pub use entry_runner::EditorApplicationComposition;
pub use entry_runner::EntryRunner;
#[cfg(feature = "diagnostic-log")]
pub use entry_runner::{
    HeadlessController, HeadlessHostError, HeadlessRunReport, HeadlessStopReason,
};
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
pub use product_shutdown::{
    retry_product_cleanup_until, ProductCloseError, ProductCompositionFailure,
};
pub use product_shutdown::{
    ProductExitClass, ProductProcessExitCode, ProductTerminalOutcome, ProductTerminalPrimary,
    ProductTerminalReceipt, ProductTerminalSecondary, ProductTerminalStatus,
    PRODUCT_TERMINAL_RECEIPT_SCHEMA_VERSION,
};
