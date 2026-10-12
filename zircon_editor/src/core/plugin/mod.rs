//! Editor-plugin descriptor, lifecycle, and catalog boundaries.

pub(crate) fn project_native_plugin_directory(
    project_root: &std::path::Path,
) -> std::path::PathBuf {
    project_root.join("zircon_plugins")
}

#[path = "admission.rs"]
mod admission;
#[path = "capability_report.rs"]
mod capability_report;
#[path = "catalog.rs"]
mod catalog;
#[path = "catalog_gen.rs"]
mod catalog_gen;
#[path = "catalog_snapshot.rs"]
mod catalog_snapshot;
#[path = "catalog_store.rs"]
mod catalog_store;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "extension_catalog_report.rs"]
mod extension_catalog_report;
#[path = "extension_materialization.rs"]
mod extension_materialization;
#[path = "isolation.rs"]
mod isolation;
#[path = "lifecycle_message_bridge.rs"]
mod lifecycle_message_bridge;
#[path = "manager.rs"]
mod manager;
#[path = "materializer.rs"]
mod materializer;
#[path = "panel_source.rs"]
mod panel_source;
#[path = "phases.rs"]
mod phases;
#[path = "projection.rs"]
mod projection;
#[path = "registration.rs"]
mod registration;
#[path = "sdk/mod.rs"]
pub mod sdk;

// Catalog read and validation surface.
pub use admission::EditorPluginCatalogAdmissionError;
pub use capability_report::EditorCapabilityReport;
pub(crate) use catalog::EditorPluginCatalog;
pub use catalog::EditorPluginHandle;
pub use catalog_snapshot::EditorPluginCatalogSnapshot;
pub use extension_catalog_report::EditorExtensionCatalogReport;
pub use isolation::{run_editor_plugin_boundary, EditorPluginBoundaryFailure};
pub use lifecycle_message_bridge::{
    EditorPluginLifecycleMessageBridge, EditorPluginLifecycleMessagePumpReport,
};
pub use manager::EditorPluginManager;
pub(crate) use manager::ProjectPluginRegistrationCloseReceipt;
pub use manager::{
    EditorPluginDiscovery, EditorPluginDiscoveryError, EditorPluginSource, EditorPluginState,
    EditorPluginTransitionError,
};
pub(crate) use materializer::materialize_serialized_native_contribution_batch;
pub use materializer::{
    materialize_serialized_contribution_batch, SerializedContributionMaterializationError,
};
pub use panel_source::{EditorPluginPanelRow, EditorPluginPanelSource};
pub use phases::EditorPluginLoadingPhase;
pub use projection::{EditorPluginCatalogEntry, EditorPluginCatalogProjection};

// Plugin declaration and lifecycle surface.
pub use descriptor::{EditorPlugin, EditorPluginDescriptor};
pub use registration::EditorPluginRegistrationReport;
