//! Entry runners that bootstrap the core runtime and host editor/runtime shells.

#[cfg(feature = "dev-dynamic-linking")]
// EXEMPT(GEN-Q7): feature-gated dev dynamic linking keeps this intentionally unused crate import available.
#[allow(unused_imports, clippy::single_component_path_imports)]
use zr_runtime_dev_dylib as _;

mod entry;
pub use entry::{retry_product_cleanup_until, ProductCloseError, ProductCompositionFailure};
pub mod plugins;
pub mod prelude;
#[cfg(feature = "platform-window")]
mod reference_cpu_presenter;

#[cfg(feature = "target-editor-host")]
pub use entry::EditorApplicationComposition;
pub use entry::{
    bootstrap_export_runtime, bootstrap_export_runtime_with_native_plugins_from_export_root,
    discover_export_root, ExportRuntimeBootstrapConfig,
    ExportRuntimePluginFeatureRegistrationProvider, ExportRuntimePluginRegistrationProvider,
};
pub use entry::{
    first_party_runtime_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_runtime_profile,
};
pub use entry::{
    EntryConfig, EntryProfile, EntryRunner, ProductArtifactDeliveryStatus, ProductArtifactKind,
    ProductArtifactManifest, ProductCapabilityRequirement, ProductComposition,
    ProductCompositionRequest, ProductConfigSource, ProductConfigSourceSet, ProductEntryKind,
    ProductExitClass, ProductHostCapabilityPolicy, ProductHostConfigError,
    ProductHostConfigProvenance, ProductPlatformClass, ProductProcessExitCode,
    ProductRoleDescriptor, ProductRoleRequest, ProductRunnerKind, ProductRuntimeLinkage,
    ProductShutdownPolicy, ProductTerminalOutcome, ProductTerminalPrimary, ProductTerminalReceipt,
    ProductTerminalSecondary, ProductTerminalStatus, ResolvedProductHostConfig,
    PRODUCT_TERMINAL_RECEIPT_SCHEMA_VERSION,
};
pub use entry::{EntryModuleSelection, EntryModuleSelectionReport, EntryRunMode};
#[cfg(feature = "diagnostic-log")]
pub use entry::{HeadlessController, HeadlessHostError, HeadlessRunReport, HeadlessStopReason};
pub use plugins::{
    DefaultPlugins, DevPlugins, HeadlessPlugins, MinimalPlugins, PluginGroup, PluginGroupBuilder,
    PluginGroupError, ResolvedPluginGroup,
};

pub use entry::{retry_runtime_startup_cleanup, RuntimeSessionCreateFailure};

#[cfg(test)]
mod tests;
