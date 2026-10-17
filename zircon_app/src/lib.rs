//! Entry runners that bootstrap the core runtime and host editor/runtime shells.

#[cfg(all(feature = "native-host", target_arch = "wasm32"))]
compile_error!("native-host requires a native target; disable default features for browser builds");

#[cfg(feature = "dev-dynamic-linking")]
// EXEMPT(GEN-Q7): feature-gated dev dynamic linking keeps this intentionally unused crate import available.
#[allow(unused_imports, clippy::single_component_path_imports)]
use zr_runtime_dev_dylib as _;

#[cfg(feature = "native-host")]
#[path = "entry/mod.rs"]
mod entry;
#[cfg(feature = "native-host")]
pub use entry::{retry_product_cleanup_until, ProductCloseError, ProductCompositionFailure};
#[cfg(feature = "native-host")]
#[path = "plugins/mod.rs"]
pub mod plugins;
#[path = "prelude.rs"]
pub mod prelude;
#[cfg(all(feature = "native-host", feature = "platform-window"))]
#[path = "reference_cpu_presenter.rs"]
mod reference_cpu_presenter;

#[cfg(feature = "target-editor-host")]
#[cfg(feature = "native-host")]
pub use entry::EditorApplicationComposition;
#[cfg(feature = "native-host")]
pub use entry::{
    bootstrap_export_runtime, bootstrap_export_runtime_with_native_plugins_from_export_root,
    discover_export_root, ExportRuntimeBootstrapConfig,
    ExportRuntimePluginFeatureRegistrationProvider, ExportRuntimePluginRegistrationProvider,
};
#[cfg(feature = "native-host")]
pub use entry::{
    first_party_runtime_plugin_registrations_for_config,
    first_party_runtime_plugin_registrations_for_manifest,
    first_party_runtime_plugin_registrations_for_runtime_profile,
};
#[cfg(feature = "native-host")]
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
#[cfg(feature = "native-host")]
pub use entry::{EntryModuleSelection, EntryModuleSelectionReport, EntryRunMode};
#[cfg(feature = "diagnostic-log")]
#[cfg(feature = "native-host")]
pub use entry::{HeadlessController, HeadlessHostError, HeadlessRunReport, HeadlessStopReason};
#[cfg(feature = "native-host")]
pub use plugins::{
    DefaultPlugins, DevPlugins, HeadlessPlugins, MinimalPlugins, PluginGroup, PluginGroupBuilder,
    PluginGroupError, ResolvedPluginGroup,
};

#[cfg(feature = "native-host")]
pub use entry::{retry_runtime_startup_cleanup, RuntimeSessionCreateFailure};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

#[cfg(all(
    feature = "target-web",
    target_arch = "wasm32",
    not(feature = "native-host")
))]
pub mod web;
